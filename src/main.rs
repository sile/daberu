use daberu::command::Command;
use orfail::OrFail;

fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    let mut command = Command {
        openai_api_key: noargs::opt("openai-api-key")
            .ty("STRING")
            .env("OPENAI_API_KEY")
            .doc("OpenAI API key")
            .take(&mut args)
            .parse_if_present()?,
        anthropic_api_key: noargs::opt("anthropic-api-key")
            .ty("STRING")
            .env("ANTHROPIC_API_KEY")
            .doc("Anthropic API key")
            .take(&mut args)
            .parse_if_present()?,
        log: noargs::opt("log")
            .short('l')
            .ty("PATH")
            .env("DABERU_LOG_PATH")
            .doc(concat!(
                "Path to log file for saving the last conversation\n",
                "\n",
                "If the file already exists, its contents are truncated\n",
                "unless `--continue` flag are specified",
            ))
            .take(&mut args)
            .parse_if_present()?,
        continue_from_log: noargs::flag("continue")
            .short('c')
            .doc(concat!(
                "Continue conversation from the existing log file ",
                "instead of truncating it"
            ))
            .take(&mut args)
            .is_present(),
        models: noargs::opt("model")
            .short('m')
            .ty("[PROVIDER:]MODEL_NAME")
            .default("gpt-4o")
            .env("DABERU_MODEL")
            .doc("Model name")
            .take(&mut args)
            .parse::<String>()?
            .split(',')
            .map(String::from)
            .collect(),
        system: noargs::opt("system")
            .short('s')
            .ty("STRING")
            .doc("System message")
            .take(&mut args)
            .parse_if_present()?,
        gist: noargs::opt("gist")
            .ty("new | EXISTING_GIST_ID")
            .doc(concat!(
                "Save the output to GitHub Gist\n",
                "\n",
                "If `EXISTING_GIST_ID` is specified,\n",
                "load the log from the Gist entry and update the entry"
            ))
            .take(&mut args)
            .parse_if_present()?,
        resources: Vec::new(),
    };

    while let Some(r) = noargs::opt("resource")
        .short('r')
        .doc(concat!(
            "File path to content that will be used as a resource for the conversion\n",
            "\n",
            "This option can be specified multiple times"
        ))
        .take(&mut args)
        .parse_if_present()?
    {
        command.resources.push(r);
    }

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    command.run().or_fail()?;
    Ok(())
}
