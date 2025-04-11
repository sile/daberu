use std::{path::PathBuf, str::FromStr};

use nojson::DisplayJson;
use orfail::OrFail;

#[derive(Debug)]
pub enum Resource {
    File(FileResource),
    Shell(ShellResource),
}

impl Resource {
    pub fn truncate(&mut self, mut n: usize) {
        match self {
            Resource::File(r) => {
                if r.content.len() <= n {
                    return;
                }
                while !r.content.is_char_boundary(n) {
                    n -= 1;
                }
                eprintln!(
                    "[WARNING] File resource ({}) exceeds size limit (truncated): size={}, limit={}",
                    r.path.display(),
                    r.content.len(),
                    n
                );
                r.content.truncate(n);
            }
            Resource::Shell(r) => {
                if r.output.len() <= n {
                    return;
                }
                while !r.output.is_char_boundary(n) {
                    n -= 1;
                }
                eprintln!(
                    "[WARNING] Shell resource (`{}`) exceeds size limit (truncated): size={}, limit={}",
                    r.command.len(),
                    r.output.len(),
                    n
                );
                r.output.truncate(n);
            }
        }
    }
}

impl FromStr for Resource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(command) = s.strip_prefix("sh:") {
            ShellResource::new(command).map(Self::Shell)
        } else if let Some(path) = s.strip_prefix("file:") {
            FileResource::new(PathBuf::from(path)).map(Self::File)
        } else {
            FileResource::new(PathBuf::from(s)).map(Self::File)
        }
        .map_err(|e| e.message)
    }
}

impl DisplayJson for Resource {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        match self {
            Resource::File(r) => f.object(|f| {
                f.member("type", "file")?;
                f.member("path", &r.path)?;
                f.member("content", &r.content)
            }),
            Resource::Shell(r) => f.object(|f| {
                f.member("type", "shell")?;
                f.member("command", &r.command)?;
                f.member("output", &r.output)
            }),
        }
    }
}

#[derive(Debug)]
pub struct FileResource {
    path: PathBuf,
    content: String,
}

impl FileResource {
    fn new(path: PathBuf) -> orfail::Result<Self> {
        let content = std::fs::read_to_string(&path)
            .or_fail_with(|e| format!("failed to read resource file {}: {e}", path.display()))?;
        Ok(Self { path, content })
    }
}

#[derive(Debug)]
pub struct ShellResource {
    command: String,
    output: String,
}

impl ShellResource {
    fn new(command: &str) -> orfail::Result<Self> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .or_fail_with(|e| format!("failed to execute shell command {command:?}: {e}"))?;
        if !output.status.success() {
            return Err(orfail::Failure::new(format!(
                "failed to execute shell command {command:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(Self {
            command: command.to_owned(),
            output: String::from_utf8(output.stdout).or_fail_with(|e| {
                format!("the output of shell command {command:?} is not a UTF-8 string: {e}")
            })?,
        })
    }
}
