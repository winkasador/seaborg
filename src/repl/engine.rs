use std::{io::{BufRead, BufReader, BufWriter, Write}, process::{Child, ChildStdin, ChildStdout, Command, Stdio}};

pub struct Engine {
    _process: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>
}

impl Engine {
    pub fn start(path: &str) -> Self {
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("$: Could not start engine");
        let stdin = BufWriter::new(process.stdin.take().unwrap());
        let stdout = BufReader::new(process.stdout.take().unwrap());

        Engine { _process: process, stdin, stdout }
    }

    pub fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{}", cmd).unwrap();
        self.stdin.flush().unwrap();
    }

    pub fn read_line(&mut self) -> String {
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        line.trim().to_string()
    }

    pub fn read_until(&mut self, predicate: impl Fn(&str) -> bool) -> Vec<String> {
        let mut lines = Vec::new();
        loop {
            let line = self.read_line();
            lines.push(line.clone());
            if predicate(&line) {
                break;
            }
        }
        lines
    }

    pub fn shutdown(&mut self) {
        self.send("quit");

        let r = self._process.kill();
        if r.is_err() {
            println!("$: Error during engine shutdown. {}", r.unwrap_err())
        }
    }
}