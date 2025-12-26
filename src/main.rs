use std::io::Read;

use daberu::{
    command::Command,
    resource::{FileResource, Resource, ShellResource},
};
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

    let enable_subcommand = noargs::flag("ext")
        .short('x')
        .doc("Enable extended subcommands")
        .take(&mut args)
        .is_present();
    if enable_subcommand {
        if noargs::cmd("last")
            .doc("Display the last message from the conversation log")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_last::run(&mut args)?;
        } else if noargs::cmd("list-skills")
            .doc("List available skills")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_list_skills::run(&mut args)?;
        } else if noargs::cmd("create-skill")
            .doc("Create a new skill from a directory")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_create_skill::run(&mut args)?;
        } else if noargs::cmd("get-skill")
            .doc("Retrieve details about a specific skill")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_get_skill::run(&mut args)?;
        } else if noargs::cmd("delete-skill")
            .doc("Delete a skill and all its versions")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_delete_skill::run(&mut args)?;
        } else if noargs::cmd("list-files")
            .doc("List all files")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_list_files::run(&mut args)?;
        } else if noargs::cmd("get-file-metadata")
            .doc("Get file metadata")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_get_file_metadata::run(&mut args)?;
        } else if noargs::cmd("get-file")
            .doc("Download file content")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_get_file::run(&mut args)?;
        } else if noargs::cmd("delete-file")
            .doc("Delete a file")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_delete_file::run(&mut args)?;
        } else if noargs::cmd("clean-files")
            .doc("Delete all files")
            .take(&mut args)
            .is_present()
        {
            daberu::subcommand_clean_files::run(&mut args)?;
        }

        if let Some(help) = args.finish()? {
            print!("{help}");
        }
        return Ok(());
    }

    let config = noargs::opt("config-file")
        .ty("PATH")
        .env("DABERU_CONFIG_FILE")
        .doc("Path to configuration JSONC file")
        .take(&mut args)
        .present_and_then(|a| daberu::config::Config::load(a.value()))?
        .unwrap_or_default();
    let mut command = Command {
        config,
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
        enable_agents_md: noargs::flag("enable-agents-md")
            .short('a')
            .env("DABERU_ENABLE_AGENTS_MD")
            .doc(concat!(
                "Automatically load AGENTS.md or CLAUDE.md as a resource\n",
                "\n",
                "If the file exists in the current directory, it will be ",
                "prepended to the resources list"
            ))
            .take(&mut args)
            .is_present(),
        model: noargs::opt("model")
            .short('m')
            .ty("MODEL_NAME")
            .default("claude-sonnet-4-5")
            .env("DABERU_MODEL")
            .doc("Model name")
            .take(&mut args)
            .then(|a| a.value().parse())?,
        system: noargs::opt("system")
            .short('s')
            .ty("STRING")
            .doc("System message")
            .take(&mut args)
            .present_and_then(|a| a.value().parse())?,
        resources: std::iter::from_fn(|| {
            noargs::opt("resource")
                .short('r')
                .ty("PATH")
                .doc(concat!(
                    "File path to be used as a resource for the conversion\n",
                    "\n",
                    "This option can be specified multiple times"
                ))
                .take(&mut args)
                .present_and_then(|a| FileResource::new(a.value()).map(Resource::File))
                .transpose()
        })
        .collect::<Result<_, _>>()?,
        skill_ids: std::iter::from_fn(|| {
            noargs::opt("skill")
                .short('k')
                .ty("SKILL_ID")
                .doc(concat!(
                    "Skill ID or preset name to use (e.g., 'pptx', 'skill_01ABC...')\n",
                    "\n",
                    "Skill IDs starting with 'skill_' are treated as custom skills,\n",
                    "others are treated as Anthropic-provided skills.\n",
                    "This option can be specified multiple times"
                ))
                .take(&mut args)
                .present_and_then(|a| a.value().parse())
                .transpose()
        })
        .collect::<Result<_, _>>()?,
    };
    command.resolve_skill_presets();

    if command.enable_agents_md {
        if let Ok(r) = FileResource::new("AGENTS.md") {
            command.resources.insert(0, Resource::File(r));
        } else if let Ok(r) = FileResource::new("CLAUDE.md") {
            command.resources.insert(0, Resource::File(r));
        }
    }

    while let Some(a) = noargs::opt("shell-command")
        .short('e')
        .ty("COMMAND")
        .doc(concat!(
            "Shell command to be used as a resource for the conversion\n",
            "\n",
            "This option can be specified multiple times"
        ))
        .take(&mut args)
        .present()
    {
        command.resources.push(Resource::Shell(ShellResource::new(
            &command.config.shell_executable,
            a.value(),
        )));
    }
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).or_fail()?;
    (!input.is_empty()).or_fail_with(|()| "empty input message".to_owned())?;

    for r in &mut command.resources {
        r.handle_input(&input).or_fail()?;
        r.truncate(command.config.resource_size_limit);
    }

    command.run(input).or_fail()?;
    Ok(())
}
