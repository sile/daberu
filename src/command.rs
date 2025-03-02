use std::path::PathBuf;

use orfail::OrFail;

use crate::{chat_gpt::ChatGpt, claude::Claude, message::MessageLog};

#[derive(Debug, clap::Args)]
pub struct Command {
    /// OpenAI API key.
    #[arg(
        long,
        value_name = "OPENAI_API_KEY",
        env = "OPENAI_API_KEY",
        hide_env_values = true
    )]
    pub openai_api_key: Option<String>,

    /// Anthropic API key.
    #[arg(
        long,
        value_name = "ANTHROPIC_API_KEY",
        env = "ANTHROPIC_API_KEY",
        hide_env_values = true
    )]
    pub anthropic_api_key: Option<String>,

    /// Path to log file for saving conversation history. If the file already exists,
    /// its contents are included in subsequent conversations.
    #[arg(long)]
    pub log: Option<PathBuf>,

    /// Specifies a path for a "one-shot" conversation log file which
    /// will overwrite any existing file content upon opening.
    ///
    /// If `--log` is specified, this option will be ignored.
    #[arg(long, value_name = "DABERU_ONESHOT_LOG_PATH")]
    pub oneshot_log: Option<PathBuf>,

    /// Model name.
    #[arg(long, env = "DABERU_MODEL", default_value = "gpt-4o")]
    pub model: String,

    /// System message.
    #[arg(long, value_name = "SYSTEM_MESSAGE", env = "DABERU_SYSTEM_MESSAGE")]
    pub system: Option<String>,
}

impl Command {
    pub fn run(self) -> orfail::Result<()> {
        self.check_api_key().or_fail()?;

        let mut log = self
            .log_file_path()
            .filter(|path| path.exists())
            .map(MessageLog::load)
            .transpose()
            .or_fail()?
            .unwrap_or_default();
        if self.log.is_none() && self.oneshot_log.is_some() {
            log.messages.clear();
        }
        if let Some(system) = &self.system {
            log.set_system_message_if_empty(system);
        }
        log.read_input().or_fail()?;

        let output = if self.model.starts_with("gpt") {
            let c = ChatGpt::new(&self).or_fail()?;
            c.run(&log).or_fail()?
        } else if self.model.starts_with("claude") {
            let c = Claude::new(&self).or_fail()?;
            c.run(&log).or_fail()?
        } else {
            unreachable!()
        };

        log.messages.push(output);
        if let Some(path) = self.log_file_path() {
            log.save(path).or_fail()?;
        }

        Ok(())
    }

    fn log_file_path(&self) -> Option<&PathBuf> {
        self.log.as_ref().or(self.oneshot_log.as_ref())
    }

    fn check_api_key(&self) -> orfail::Result<()> {
        if self.model.starts_with("gpt") {
            self.openai_api_key
                .is_some()
                .or_fail_with(|()| "OpenAI API key is not specified".to_owned())?;
        } else if self.model.starts_with("claude") {
            self.anthropic_api_key
                .is_some()
                .or_fail_with(|()| "Anthropic API key is not specified".to_owned())?;
        } else {
            return Err(orfail::Failure::new("unknown model"));
        }
        Ok(())
    }
}
