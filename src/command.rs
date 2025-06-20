use std::path::PathBuf;

use orfail::OrFail;

use crate::{chat_gpt::ChatGpt, claude::Claude, gist, message::MessageLog, resource::Resource};

#[derive(Debug)]
pub struct Command {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub log: Option<PathBuf>,
    pub continue_from_log: bool,
    pub model: String,
    pub system: Option<String>,
    pub gist: Option<String>,
    pub resources: Vec<Resource>,
    pub resource_size_limit: usize,
}

impl Command {
    pub fn run(self, input: String) -> orfail::Result<()> {
        self.check_api_key().or_fail()?;

        let mut gist_offset = 0;
        let mut log = self
            .log
            .as_ref()
            .filter(|path| path.exists())
            .map(MessageLog::load)
            .transpose()
            .or_fail()?
            .unwrap_or_default();
        if !self.continue_from_log {
            log.messages.clear();
        }
        if let Some(id) = self.gist.as_ref().filter(|id| *id != "new") {
            log = gist::load(id).or_fail()?;
            gist_offset = log.messages.len();
        }
        if let Some(system) = &self.system {
            log.set_system_message_if_empty(system);
        }
        log.read_input(input, &self.resources).or_fail()?;

        let model = &self.model;
        let output = if model.starts_with("gpt") || model.starts_with("openai:") {
            let model = model.strip_prefix("openai:").unwrap_or(model).to_owned();
            let c = ChatGpt::new(&self, model).or_fail()?;
            let log = log.strip_model_name();
            c.run(&log).or_fail()?
        } else if model.starts_with("claude") || model.starts_with("anthropic:") {
            let model = model.strip_prefix("anthropic:").unwrap_or(model).to_owned();
            let c = Claude::new(&self, model).or_fail()?;
            let log = log.strip_model_name();
            c.run(&log).or_fail()?
        } else {
            unreachable!()
        };
        log.messages.push(output);

        if let Some(path) = self.log {
            log.save(path).or_fail()?;
        }
        match self.gist.as_deref() {
            Some("new") => {
                eprintln!();
                gist::create(&log).or_fail()?;
            }
            Some(id) => {
                eprintln!();
                gist::update(id, &log, gist_offset).or_fail()?;
            }
            None => {}
        }

        Ok(())
    }

    fn check_api_key(&self) -> orfail::Result<()> {
        let model = &self.model;
        if model.starts_with("gpt") || model.starts_with("openai:") {
            self.openai_api_key
                .is_some()
                .or_fail_with(|()| "OpenAI API key is not specified".to_owned())?;
        } else if model.starts_with("claude") || model.starts_with("anthropic:") {
            self.anthropic_api_key
                .is_some()
                .or_fail_with(|()| "Anthropic API key is not specified".to_owned())?;
        } else {
            return Err(orfail::Failure::new(format!("unknown model: {model}")));
        }
        Ok(())
    }
}
