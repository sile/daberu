use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("FOOBARBAZ")
        .take(args)
        .then(|a| a.value().parse())?;

    // TODO: opt
    let display_title: String = noargs::arg("DISPLAY_TITLE")
        .example("My Skill")
        .doc("Display title for the skill")
        .take(args)
        .then(|a| a.value().parse())?;

    let skill_file: String = noargs::arg("SKILL_DIR")
        .example("/path/to/skill/")
        .doc("Path to skill directory to upload")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(());
    }

    // Verify the skill file exists
    std::fs::metadata(&skill_file).or_fail()?;

    // Use CurlRequest for multipart form data upload
    let response = crate::curl::CurlRequest::new("https://api.anthropic.com/v1/skills")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .post_multipart(vec![
            ("display_title".to_string(), display_title),
            ("files".to_string(), format!("@{}", skill_file)),
        ])
        .or_fail()?;

    let response = response.check_success().or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}
