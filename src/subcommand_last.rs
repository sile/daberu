use std::path::PathBuf;

use crate::message::MessageLog;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let log: PathBuf = noargs::opt("log")
        .short('l')
        .ty("PATH")
        .env("DABERU_LOG_PATH")
        .doc("Path to log file containing the conversation history")
        .take(args)
        .then(|a| a.value().parse())?;
    let log = MessageLog::load(log)?;
    if let Some(m) = log.messages.last() {
        println!("{}", m.content);
    }
    Ok(())
}
