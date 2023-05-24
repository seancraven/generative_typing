use console::{style, Key, Term};
use core::fmt;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::error::Error;
use std::io::{self, BufRead, Lines};
use std::iter::Skip;
use std::str::Chars;

fn first_non_space(line: &str, term: &Term) -> Result<usize, LineError> {
    for (i, char) in line.trim_end().chars().enumerate() {
        if char == ' ' {
            term.move_cursor_right(1)?;
        } else {
            return Ok(i);
        }
    }
    return Ok(line.len());
}

/// Function that handles the user input and displays the output to typed.
pub fn type_line(line: &str, term: &Term, window_size: usize) -> Result<Analytics, LineError> {
    term.move_cursor_up(window_size + 1)?;
    let mut line_iter = line.trim_end().chars().skip(first_non_space(line, term)?);
    let analytics = RefCell::new(Analytics::new(0, 0, 0));

    loop {
        let (user_input, invalid_keystrokes) = Input::is_valid(term)?;
        analytics.borrow_mut().errors += invalid_keystrokes;
        if let Some(target_char) = line_iter.next() {
            // Record that the next item is seen.
            // See what to do:
            match user_input {
                // Simple happy path.
                Input::Char(c) => {
                    analytics.borrow_mut().errors += write_char(term, target_char, c)?;
                    analytics.borrow_mut().total_input_chars += 1;
                    analytics.borrow_mut().line_length += 1;
                }
                // Tab counts as four sequential enters.
                // First enter does not add to the line position as this
                Input::Tab => {
                    analytics.borrow_mut().errors += write_char(term, target_char, ' ')?;
                    analytics.borrow_mut().total_input_chars += 1;
                    analytics.borrow_mut().line_length += 1;
                    for _ in 0..3 {
                        if let Some(c) = line_iter.next() {
                            analytics.borrow_mut().errors += write_char(term, c, ' ')?;
                            analytics.borrow_mut().total_input_chars += 1;
                            analytics.borrow_mut().line_length += 1;
                        } else {
                            break;
                        }
                    }
                }
                Input::Backspace => {
                    line_iter = backspace(term, &analytics, line)?;
                }
                // Pressing enter in the middle of a line just counts as a mistake.
                Input::Enter => {
                    analytics.borrow_mut().errors += 1;
                    analytics.borrow_mut().total_input_chars += 1;
                }
                Input::Esc => return Err(LineError::Esc(*analytics.borrow())),
            }
        } else {
            // Handle an empty line. Or Line end.
            match user_input {
                Input::Enter => {
                    analytics.borrow_mut().total_input_chars += 1;
                    analytics.borrow_mut().line_length += 1;
                    break;
                }
                Input::Backspace => {
                    line_iter = backspace(term, &analytics, line)?;
                }
                Input::Char(c) => analytics.borrow_mut().errors += write_char(term, ' ', c)?,
                Input::Tab => analytics.borrow_mut().errors += 1,
                Input::Esc => return Err(LineError::Esc(*analytics.borrow())),
            }
        }
    }
    return Ok(*analytics.borrow());
}
/// Function to perform a backspace on the terminal.
/// Moves the current position back one and increases the
/// total input char.
fn backspace<'a>(
    term: &'a Term,
    analytics: &RefCell<Analytics>,
    line: &'a str,
) -> Result<Skip<Chars<'a>>, io::Error> {
    if analytics.borrow_mut().line_length > 0 {
        term.move_cursor_left(1)?;
        analytics.borrow_mut().line_length -= 1;
        analytics.borrow_mut().total_input_chars += 1;
    }
    Ok(line
        .chars()
        .skip(analytics.borrow_mut().line_length as usize))
}
/// Writes a character to the terminal color coded for correctness.
/// Returns 1 if error, else 0
fn write_char(term: &Term, target_char: char, user_char: char) -> Result<u64, io::Error> {
    let mut error = 0;
    // TODO: Add a ¬ character, to show return shouls have been pressed.
    if target_char == user_char {
        term.write_str(&format!("{}", style(user_char).green()))?;
    } else {
        term.write_str(&format!("{}", style(target_char).red()))?;
        error += 1
    }
    return Ok(error);
}

#[derive(Debug, PartialEq)]
enum Input {
    Char(char),
    Backspace,
    Tab,
    Enter,
    Esc,
}
/// Valid entries are any character, space, backspace, tab, enter and esc.
/// Function takes in a reference to the terminal and returns the
/// key type and the number of invalid keystrokes made before entering
/// a valid key.
impl Input {
    fn is_valid(term: &Term) -> Result<(Input, u64), io::Error> {
        let mut invalid_keystrokes = 0;
        let mut in_key = None;
        while None == in_key {
            in_key = match term.read_key()? {
                Key::Char(c) => Some(Input::Char(c)),
                Key::Backspace => Some(Input::Backspace),
                Key::Tab => Some(Input::Tab),
                Key::Enter => Some(Input::Enter),
                Key::Escape => Some(Input::Esc),
                _ => None,
            };

            invalid_keystrokes += 1;
        }
        invalid_keystrokes -= 1;
        if let Some(in_key) = in_key {
            return Ok((in_key, invalid_keystrokes));
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to read key"));
        }
    }
}

/// Sliding window iterator over a buffer, returning a window of lines.
///
#[derive(Debug)]
pub struct LinesGenerator<T: Sized + BufRead> {
    // Light generic to allow for different types of readers.
    current_window: LinkedList<String>,
    lines_iter: Lines<T>,
}

impl<T: Sized + BufRead> LinesGenerator<T> {
    /// Create a new FileLinesGenerator. With the first window
    /// populate.
    ///
    /// # Arguments
    /// * `file` - File to read from
    /// * `window_size` - Number of lines to show to the user
    ///
    /// # Returns
    /// * `FileLinesGenerator` - Struct to read lines from file
    pub fn new(reader: T, window_size: usize) -> Self {
        let mut lines_iter = reader.lines();
        let mut current_window = LinkedList::new();
        current_window.push_front("".to_string());
        for _ in 0..window_size {
            match lines_iter.next() {
                Some(line) => {
                    if let Ok(line) = line {
                        current_window.push_back(line);
                    }
                }
                None => break,
            }
        }
        return Self {
            current_window,
            lines_iter,
        };
    }
}
impl<T: Sized + BufRead> Iterator for LinesGenerator<T> {
    type Item = LinkedList<String>;
    fn next(&mut self) -> Option<Self::Item> {
        // returns slice of current_chunk.
        // If current_chunk will be_empty, load next chunk before
        // returning.
        self.current_window.pop_front();
        match self.lines_iter.next() {
            Some(line) => {
                if let Ok(line) = line {
                    self.current_window.push_back(line);
                    return Some(self.current_window.clone());
                } else {
                    return Some(self.current_window.clone());
                }
            }
            None => {
                println!("{:?}", self.current_window.clone());
                if self.current_window.len() > 0 {
                    return Some(self.current_window.clone());
                } else {
                    return None;
                }
            }
        }
    }
}
/// Struct to hold analytics about the user's performance.
#[derive(Debug, Clone, Copy)]
pub struct Analytics {
    pub errors: u64,
    pub total_input_chars: u64,
    pub line_length: u64,
}
impl Analytics {
    pub fn new(errors: u64, total_input_chars: u64, line_length: u64) -> Analytics {
        Analytics {
            errors,
            total_input_chars,
            line_length,
        }
    }
}
#[derive(Debug)]
pub enum LineError {
    Io(io::Error),
    Esc(Analytics), //(errors, total_entered, string_len)
}
impl From<io::Error> for LineError {
    fn from(err: io::Error) -> LineError {
        LineError::Io(err)
    }
}
impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LineError::Io(e) => e.fmt(f),
            LineError::Esc(a) => write!(
                f,
                "Escape with\nErrors: {}\nTotal:{}\nString Lenght{}",
                a.errors, a.total_input_chars, a.line_length
            ),
        }
    }
}
/// Wrapper for io::Errror, and escape functionality.
/// enables quick exit from typing. Raise this with the
/// current data and handle this.
impl Error for LineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            LineError::Esc(_) => None,
            LineError::Io(ref e) => Some(e),
        }
    }
}
