use std::convert::Infallible;

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
            .present_and_then(|a| a.value().parse())?,
        anthropic_api_key: noargs::opt("anthropic-api-key")
            .ty("STRING")
            .env("ANTHROPIC_API_KEY")
            .doc("Anthropic API key")
            .take(&mut args)
            .present_and_then(|a| a.value().parse())?,
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
            .present_and_then(|a| a.value().parse())?,
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
            .then(|a| -> Result<_, Infallible> {
                Ok(a.value().split(',').map(String::from).collect())
            })?,
        system: noargs::opt("system")
            .short('s')
            .ty("STRING")
            .doc("System message")
            .take(&mut args)
            .present_and_then(|a| a.value().parse())?,
        gist: noargs::opt("gist")
            .ty("new | EXISTING_GIST_ID")
            .doc(concat!(
                "Save the output to GitHub Gist\n",
                "\n",
                "If `EXISTING_GIST_ID` is specified,\n",
                "load the log from the Gist entry and update the entry"
            ))
            .take(&mut args)
            .present_and_then(|a| a.value().parse())?,
        resources: std::iter::from_fn(|| {
            noargs::opt("resource")
                .short('r')
                .ty("[file:]PATH | sh:COMMAND")
                .doc(concat!(
                    "File path or command to be used as a resource for the conversion\n",
                    "\n",
                    "Prefixes:\n",
                    "- `file:PATH` - explicitly specify a file path (default if no prefix)\n",
                    "- `sh:COMMAND` - execute shell command and use its output\n",
                    "\n",
                    "This option can be specified multiple times"
                ))
                .take(&mut args)
                .present_and_then(|a| a.value().parse())
                .transpose()
        })
        .collect::<Result<_, _>>()?,
        resource_size_limit: noargs::opt("resource-size-limit")
            .short('l')
            .default("100000")
            .ty("BYTE_SIZE")
            .doc(concat!(
                "Maximum byte size per resource\n",
                "\n",
                "If a resource exceeds this limit, the remaining content will be truncated"
            ))
            .take(&mut args)
            .then(|a| a.value().parse())?,
    };
    for r in &mut command.resources {
        r.truncate(command.resource_size_limit);
    }

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    command.run().or_fail()?;
    Ok(())
}
