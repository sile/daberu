use std::collections::BTreeMap;
use std::path::Path;

use orfail::OrFail;

#[derive(Debug)]
pub struct Config {
    pub resource_size_limit: usize,
    pub shell_executable: String,
    pub resource_presets: BTreeMap<String, Vec<crate::resource::ResourceSpec>>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let text = std::fs::read_to_string(path).or_fail()?;
        let (raw, _) = nojson::RawJson::parse_jsonc(&text).or_fail()?;
        Self::try_from(raw.value()).or_fail()
    }
}

impl Default for Config {
    fn default() -> Self {
        let text = include_str!("../configs/default.jsonc");
        let (raw, _) = nojson::RawJson::parse_jsonc(text).expect("bug");
        Self::try_from(raw.value()).expect("bug")
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Config {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let resource_size_limit = value.to_member("resource_size_limit")?.required()?;
        let shell_executable = value.to_member("shell_executable")?.required()?;
        let resource_presets = value.to_member("resource_presets")?.required()?;

        Ok(Self {
            resource_size_limit: resource_size_limit.try_into()?,
            shell_executable: shell_executable.try_into()?,
            resource_presets: resource_presets.try_into()?,
        })
    }
}
