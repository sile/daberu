use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("FOOBARBAZ")
        .take(args)
        .then(|a| a.value().parse())?;
    let skill_id: String = noargs::arg("SKILL_ID")
        .example("skill_foo")
        .doc("ID of the skill to delete")
        .take(args)
        .then(|a| a.value().parse())?;
    if args.metadata().help_mode {
        return Ok(());
    }

    // First, list all versions of the skill
    let versions_response = crate::curl::CurlRequest::new(format!(
        "https://api.anthropic.com/v1/skills/{skill_id}/versions",
    ))
    .header("anthropic-version", "2023-06-01")
    .header("anthropic-beta", "skills-2025-10-02")
    .header("X-Api-Key", &api_key)
    .get()
    .or_fail()?
    .into_json()
    .or_fail()?;

    // Delete each version
    for version_entry in versions_response
        .value()
        .to_member("data")
        .or_fail()?
        .required()
        .or_fail()?
        .to_array()
        .or_fail()?
    {
        let version_id = version_entry
            .to_member("id")
            .or_fail()?
            .required()
            .or_fail()?
            .to_unquoted_string_str()
            .or_fail()?;
        crate::curl::CurlRequest::new(format!(
            "https://api.anthropic.com/v1/skills/{skill_id}/versions/{version_id}",
        ))
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .delete()
        .or_fail()?
        .check_success()
        .or_fail()?;
    }

    // For now, attempt to delete the skill directly
    let response =
        crate::curl::CurlRequest::new(format!("https://api.anthropic.com/v1/skills/{skill_id}"))
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "skills-2025-10-02")
            .header("X-Api-Key", &api_key)
            .delete()
            .or_fail()?
            .check_success()
            .or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}
