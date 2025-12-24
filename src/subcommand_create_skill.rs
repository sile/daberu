use std::path::{Path, PathBuf};

use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("YOUR_API_KEY")
        .take(args)
        .then(|a| a.value().parse())?;
    let skill_id: Option<String> = noargs::opt("skill-id")
        .short('s')
        .ty("STRING")
        .doc("Skill ID to update (if not provided, creates a new skill)")
        .take(args)
        .then(|a| a.value().parse())
        .ok();
    let display_title: Option<String> = noargs::opt("display-title")
        .short('t')
        .ty("STRING")
        .doc("Display title for the skill (defaults to skill directory name if not provided)")
        .take(args)
        .present_and_then(|a| a.value().parse())?;
    let skill_dir: PathBuf = noargs::arg("SKILL_DIR")
        .example("/path/to/skill/")
        .doc("Path to skill directory to upload")
        .take(args)
        .then(|a| a.value().parse())?;
    if args.metadata().help_mode {
        return Ok(());
    }

    // Verify the skill directory exists
    std::fs::metadata(&skill_dir).or_fail()?;
    (skill_dir.is_dir()).or_fail_with(|_| format!("{} is not a directory", skill_dir.display()))?;

    // Verify SKILL.md exists
    skill_dir.join("SKILL.md").exists().or_fail_with(|_| {
        format!(
            "SKILL.md not found in skill directory: {}",
            skill_dir.display()
        )
    })?;

    let display_title = if let Some(title) = display_title {
        title
    } else {
        skill_dir
            .file_name()
            .or_fail()?
            .to_string_lossy()
            .to_string()
    };
    let mut form_fields = vec![("display_title".to_string(), display_title)];
    let skill_dir = skill_dir.canonicalize().or_fail()?;

    // Collect all files from the directory
    add_files(&skill_dir, skill_dir.parent().or_fail()?, &mut form_fields)?;

    // Determine endpoint based on whether we're updating or creating
    let endpoint = if let Some(id) = &skill_id {
        format!("https://api.anthropic.com/v1/skills/{id}/versions")
    } else {
        "https://api.anthropic.com/v1/skills".to_string()
    };

    // Use CurlRequest for multipart form data upload
    let response = crate::curl::CurlRequest::new(&endpoint)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .post_multipart(form_fields)
        .or_fail()?
        .check_success()
        .or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}

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
                dbg!(format!(
                    "@{};filename={}",
                    path.to_string_lossy(),
                    relative_path
                )),
            ));
        } else if path.is_dir() {
            add_files(&path, base_path, form_fields)?;
        }
    }
    Ok(())
}
