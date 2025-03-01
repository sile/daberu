use std::path::PathBuf;

use orfail::OrFail;

use crate::{chat_gpt::ChatGpt, message::MessageLog};

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

    /// Log file path to save the conversation history. If the file already exists, the history will be considered in the next conversation.
    // TODO: Add env (DABERU_LOG_PATH)
    // TODO: --truncate_log
    #[arg(long, value_name = "LOG_FILE_PATH")]
    pub log: Option<PathBuf>,

    /// Model name.
    #[arg(long, env = "DABERU_MODEL", default_value = "gpt-4o")]
    pub model: String,

    /// System message.
    #[arg(long, value_name = "SYSTEM_MESSAGE", env = "DABERU_SYSTEM_MESSAGE")]
    pub system: Option<String>,

    /// Max tokens.
    #[arg(short = 't', long, env = "DABERU_MAX_TOKENS")]
    pub max_tokens: Option<u32>,

    // TODO: rename to "markdown"
    #[arg(short, long)]
    pub echo_input: bool,
}

impl Command {
    pub fn run(self) -> orfail::Result<()> {
        self.check_api_key().or_fail()?;

        let mut log = self
            .log
            .as_ref()
            .filter(|path| path.exists())
            .map(MessageLog::load)
            .transpose()
            .or_fail()?
            .unwrap_or_default();
        if let Some(system) = &self.system {
            log.set_system_message_if_empty(system);
        }
        log.read_input().or_fail()?;

        let output = if self.model.starts_with("gpt") {
            let c = ChatGpt::new(&self).or_fail()?;
            c.run(&log).or_fail()?
        } else if self.model.starts_with("claude") {
            todo!();
        } else {
            unreachable!()
        };

        log.messages.push(output);
        if let Some(path) = &self.log {
            log.save(path).or_fail()?;
        }

        Ok(())
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
        }
        Ok(())
    }
}
