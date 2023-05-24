use chrono::Local;
mod client;
mod typing;
use client::TypeClient;
use console::Term;
use dotenv::dotenv;
use std::io::BufReader;
use std::path::PathBuf;
use typing::{type_line, LineError, LinesGenerator};

use clap::Parser;

fn main() -> Result<(), LineError> {
    dotenv().ok();
    let args = Args::parse();
    let term = Term::stdout();
    term.write_line("Press any key to start")?;
    term.read_key()?;
    let mut errors = 0;
    let mut total = 0;
    let mut len = 0;
    // Currently only one line is displayed,
    let typeclient =
        TypeClient::new_from_env().expect("Failed to start due to lack of local .env variables");
    let window_stream = typeclient
        .start_gen("TODO: Prompt")
        .expect("Failed to connect to typing server");
    let buf = BufReader::new(&window_stream);
    let window_gen = LinesGenerator::new(buf, args.lines).into_iter();
    let start_time = Local::now();
    for window in window_gen {
        //
        let line_to_type = window.iter().next().unwrap();
        //
        term.clear_screen()?;
        term.write_line(line_to_type)?;
        window
            .iter()
            .skip(1)
            .for_each(|line| term.write_line(line).unwrap_or_default());
        //
        let line_resp = type_line(&line_to_type, &term, args.lines);
        //
        //
        match line_resp {
            Ok(a) => {
                errors += a.errors;
                total += a.total_input_chars;
                len += a.line_length;
            }
            Err(le) => match le {
                LineError::Io(io) => {
                    return Err(LineError::Io(io));
                }
                LineError::Esc(a) => {
                    errors += a.errors;
                    total += a.total_input_chars;
                    len += a.line_length;
                    break;
                }
            },
        }
    }

    let duration = (Local::now() - start_time).num_seconds() as f64 / 60.0; // time in minuites

    // handle the end of the stream.
    window_stream.shutdown(std::net::Shutdown::Both)?;
    if total != 0 {
        term.clear_screen()?;
        term.write_line(&format!("{} errors made.", errors))?;
        term.write_line(&format!(
            "{:.0}% Accuracy.",
            ((1.0 - (errors as f64 / total as f64)) * 100.0).max(0.0)
        ))?;
        term.write_line(&format!("{:.0} WPM.", (len as f64 / (5.0 * duration))))?;
        term.write_line(&format!("{} excess characters.", total as i64 - len as i64))?;
    }
    return Ok(());
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "simple.txt")]
    file_name: PathBuf,

    #[arg(short, long, default_value = "10")]
    lines: usize,
}
