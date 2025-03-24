use daberu::command::Command;
use orfail::OrFail;

fn main() -> noargs::Result<()> {
    let mut args = noargs::args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if noargs::HELP_FLAG.take(&mut args).is_present() {
        args.metadata_mut().help_mode = true;
    }

    let command = Command {
        openai_api_key: noargs::opt("openai-api-key")
            .ty("STRING")
            .env("OPENAI_API_KEY")
            .doc("OpenAI API key.")
            .take(&mut args)
            .parse_if_present()?,
        anthropic_api_key: noargs::opt("anthropic-api-key")
            .ty("STRING")
            .env("ANTHROPIC_API_KEY")
            .doc("Anthropic API key.")
            .take(&mut args)
            .parse_if_present()?,
        log: noargs::opt("log")
            .ty("PATH")
            .doc(concat!(
                "Path to log file for saving conversation history. \n",
                "If the file already exists, its contents are included\n",
                "in subsequent conversations."
            ))
            .take(&mut args)
            .parse_if_present()?,
        oneshot_log: noargs::opt("oneshot-log")
            .ty("PATH")
            .env("DABERU_ONESHOT_LOG_PATH")
            .doc(concat!(
                "Specifies a path for a \"one-shot\" conversation log file which\n",
                "will overwrite any existing file content upon opening.\n",
                "If `--log` is specified, this option will be ignored."
            ))
            .take(&mut args)
            .parse_if_present()?,
        model: noargs::opt("model")
            .short('m')
            .ty("STRING")
            .default("gpt-4o")
            .env("DABERU_MODEL")
            .doc("Model name.")
            .take(&mut args)
            .parse()?,
        system: noargs::opt("system")
            .short('s')
            .ty("STRING")
            .doc("System message.")
            .take(&mut args)
            .parse_if_present()?,
        gist: noargs::opt("gist")
            .ty("new | EXISTING_GIST_ID")
            .doc(concat!(
                "Save the output to GitHub Gist.\n",
                "If `EXISTING_GIST_ID` is specified,\n",
                "load the log from the Gist entry and update the entry."
            ))
            .take(&mut args)
            .parse_if_present()?,
    };
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    command.run().or_fail()?;
    Ok(())
}
