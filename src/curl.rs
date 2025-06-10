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
        let output = CurlResponse::from_reader(stdout)?;

        let status = child.wait().or_fail()?;
        status
            .success()
            .or_fail_with(|()| format!("curl command failed with status: {}", status))?;

        Ok(output)
    }
}

pub struct CurlResponse {
    pub status_code: u16,
    pub status_line: String,
    pub body_reader: Box<dyn Read>,
}

impl CurlResponse {
    fn from_reader<R: Read + 'static>(reader: R) -> orfail::Result<Self> {
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
        })
    }

    pub fn check_success(self) -> orfail::Result<Box<dyn Read>> {
        if self.status_code != 200 {
            // Read response body for error details
            let mut error_body = String::new();
            let mut reader = self.body_reader;
            reader.read_to_string(&mut error_body).or_fail()?;

            return Err(Failure::new(format!(
                "HTTP request failed with status {}: {}\n\nResponse body:\n{}",
                self.status_code,
                self.status_line,
                error_body.trim()
            )));
        }

        Ok(self.body_reader)
    }
}
