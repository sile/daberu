use orfail::{Failure, OrFail};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

pub struct CurlRequest {
    url: String,
    headers: Vec<(String, String)>,
    data: Option<String>,
    silent: bool,
    show_error: bool,
    no_buffer: bool,
    include_headers: bool,
}

impl CurlRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            data: None,
            silent: false,
            show_error: false,
            no_buffer: false,
            include_headers: false,
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    pub fn show_error(mut self, show_error: bool) -> Self {
        self.show_error = show_error;
        self
    }

    pub fn no_buffer(mut self, no_buffer: bool) -> Self {
        self.no_buffer = no_buffer;
        self
    }

    pub fn include_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }

    pub fn execute(self) -> orfail::Result<CurlResponse> {
        let mut cmd = std::process::Command::new("curl");
        cmd.arg(&self.url);

        // Add headers
        for (name, value) in &self.headers {
            cmd.arg("-H").arg(format!("{}: {}", name, value));
        }

        // Add data if present
        if self.data.is_some() {
            cmd.arg("-d").arg("@-"); // Read data from stdin
        }

        // Add flags
        if self.silent {
            cmd.arg("--silent");
        }
        if self.show_error {
            cmd.arg("--show-error");
        }
        if self.no_buffer {
            cmd.arg("--no-buffer");
        }
        if self.include_headers {
            cmd.arg("--include");
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .or_fail()?;

        // Write data to stdin if present
        if let Some(data) = &self.data {
            let stdin = child.stdin.take().or_fail()?;
            write!(BufWriter::new(stdin), "{}", data).or_fail()?;
        }

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
