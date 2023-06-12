## LLM typing client
When practicing my typing speed, especially in code based settings, the lack of varied and semantically correct typing apps irritatied me. So I am building one in Rust.
## The server test worktree goals.

 - [x] Get the server to send a prewritten block of text chunk by chunk.
    - [x] Start up the server
    - [x] Listen for client to send to
    - [x] Send Content piece by piece.
 - [x] Get the client to recieve a blocks and make them typable in realtime
 - [ ] Have server not crash when typing is finished and handle connection completion nicely.

