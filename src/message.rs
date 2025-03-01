use std::path::Path;

use orfail::OrFail;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MessageLog {
    pub messages: Vec<Message>,
}

impl MessageLog {
    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let file = std::fs::File::open(&path).or_fail_with(|e| {
            format!("failed to open log file {}: {e}", path.as_ref().display())
        })?;
        let this = serde_json::from_reader(file).or_fail_with(|e| {
            format!("failed to load log file {}: {e}", path.as_ref().display())
        })?;
        Ok(this)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> orfail::Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .or_fail_with(|e| {
                format!("failed to create log file {}: {e}", path.as_ref().display())
            })?;
        serde_json::to_writer(file, self).or_fail_with(|e| {
            format!("failed to save log file {}: {e}", path.as_ref().display())
        })?;
        Ok(())
    }
}
