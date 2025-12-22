use std::path::PathBuf;

use orfail::OrFail;

use crate::{claude::Claude, gist, message::MessageLog, resource::Resource};

#[derive(Debug)]
pub struct Command {
    pub anthropic_api_key: Option<String>,
    pub log: Option<PathBuf>,
    pub continue_from_log: bool,
    pub enable_agents_md: bool,
    pub model: String,
    pub system: Option<String>,
    pub gist: Option<String>,
    pub resources: Vec<Resource>,
    pub resource_size_limit: usize,
}

impl Command {
    pub fn run(self, input: String) -> orfail::Result<()> {
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

        let c = Claude::new(&self, self.model.clone()).or_fail()?;
        let output = c.run(&log.strip_model_name()).or_fail()?;
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
}
