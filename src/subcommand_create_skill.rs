use orfail::OrFail;
use std::path::Path;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("FOOBARBAZ")
        .take(args)
        .then(|a| a.value().parse())?;

    let display_title: String = noargs::arg("DISPLAY_TITLE")
        .example("My Skill")
        .doc("Display title for the skill")
        .take(args)
        .then(|a| a.value().parse())?;

    let skill_dir: String = noargs::arg("SKILL_DIR")
        .example("/path/to/skill/")
        .doc("Path to skill directory to upload")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(());
    }

    // Verify the skill directory exists
    let path = Path::new(&skill_dir);
    std::fs::metadata(&skill_dir).or_fail()?;
    (path.is_dir()).or_fail_with(|_| format!("{} is not a directory", skill_dir))?;

    // Collect all files from the directory
    let mut form_fields = vec![("display_title".to_string(), display_title)];

    // Recursively walk the directory and add all files
    fn add_files(
        dir: &Path,
        base_path: &Path,
        form_fields: &mut Vec<(String, String)>,
    ) -> orfail::Result<()> {
        for entry in std::fs::read_dir(dir).or_fail()? {
            let entry = entry.or_fail()?;
            let path = entry.path();

            if path.is_file() {
                let relative_path = path
                    .strip_prefix(base_path)
                    .or_fail()?
                    .to_string_lossy()
                    .to_string();

                form_fields.push((
                    "files[]".to_string(),
                    format!("@{};filename={}", path.to_string_lossy(), relative_path),
                ));
            } else if path.is_dir() {
                add_files(&path, base_path, form_fields)?;
            }
        }
        Ok(())
    }

    add_files(path, path, &mut form_fields)?;

    // Verify SKILL.md exists
    let skill_md_path = path.join("SKILL.md");
    skill_md_path
        .exists()
        .or_fail_with(|_| "SKILL.md not found in skill directory".to_string())?;

    // Use CurlRequest for multipart form data upload
    let response = crate::curl::CurlRequest::new("https://api.anthropic.com/v1/skills")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .post_multipart(form_fields)
        .or_fail()?;

    let response = response.check_success().or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}
