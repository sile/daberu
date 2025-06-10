use orfail::{Failure, OrFail};
use std::{
    fmt::Display,
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

pub struct CurlRequest {
    url: String,
    headers: Vec<(String, String)>,
}

impl CurlRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn post(self, data: impl Display) -> orfail::Result<CurlResponse> {
        let mut cmd = std::process::Command::new("curl");
        cmd.arg(&self.url);

        // Add headers
        for (name, value) in &self.headers {
            cmd.arg("-H").arg(format!("{}: {}", name, value));
        }

        // Add flags
        cmd.arg("-d").arg("@-"); // Read data from stdin
        cmd.arg("--silent");
        cmd.arg("--show-error");
        cmd.arg("--no-buffer");
        cmd.arg("--include");

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .or_fail()?;

        let stdin = child.stdin.take().or_fail()?;
        let mut writer = BufWriter::new(stdin);
        write!(writer, "{}", data).or_fail()?;
        writer.flush().or_fail()?;
        std::mem::drop(writer);

        let stdout = child.stdout.take().or_fail()?;

        // Return the response immediately without waiting for process completion
        let output = CurlResponse::from_reader_with_child(stdout, child)?;
        Ok(output)
    }
}

pub struct CurlResponse {
    pub status_code: u16,
    pub status_line: String,
    pub body_reader: Box<dyn BufRead>,
    child: Option<std::process::Child>,
}

impl CurlResponse {
    fn from_reader_with_child<R: Read + 'static>(
        reader: R,
        child: std::process::Child,
    ) -> orfail::Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut first_line = String::new();
        reader.read_line(&mut first_line).or_fail()?;

        // Parse HTTP status line (e.g., "HTTP/1.1 200 OK")
        first_line.starts_with("HTTP/").or_fail()?;

        // Skip remaining headers until we find the empty line
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).or_fail()?;
            if line.trim().is_empty() {
                break;
            }
        }

        let parts: Vec<&str> = first_line.split_whitespace().collect();
        (parts.len() >= 2).or_fail()?;
        let status_code: u16 = parts[1]
            .parse::<u16>()
            .or_fail_with(|_| format!("Invalid HTTP status code: {}", parts[1]))?;

        Ok(Self {
            status_code,
            status_line: first_line.trim().to_string(),
            body_reader: Box::new(reader),
            child: Some(child),
        })
    }

    pub fn check_success(mut self) -> orfail::Result<Box<dyn BufRead>> {
        if self.status_code != 200 {
            // Read response body for error details
            let mut error_body = String::new();
            let mut reader = self.body_reader;
            reader.read_to_string(&mut error_body).or_fail()?;

            // Clean up the child process
            if let Some(mut child) = self.child.take() {
                let _ = child.wait();
            }

            return Err(Failure::new(format!(
                "HTTP request failed with status {}: {}\n\nResponse body:\n{}",
                self.status_code,
                self.status_line,
                error_body.trim()
            )));
        }

        Ok(Box::new(StreamingReader {
            reader: self.body_reader,
            child: self.child,
        }))
    }
}

// Wrapper that ensures the child process is cleaned up when the reader is dropped
struct StreamingReader {
    reader: Box<dyn BufRead>,
    child: Option<std::process::Child>,
}

impl BufRead for StreamingReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

impl Read for StreamingReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Drop for StreamingReader {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.wait(); // Clean up the process when done
        }
    }
}
