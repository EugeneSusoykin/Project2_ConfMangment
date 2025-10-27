use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use thiserror::Error;

#[derive(Debug)]
pub struct AppConfig {
    pub package_name: String,
    pub repo_source: String,
    pub mode: String,
    pub ascii_tree: bool,
    pub exclude_filter: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read config file: {0}")]
    ReadError(String),

    #[error("malformed XML: {0}")]
    XmlError(String),

    #[error("missing or empty required field: {0}")]
    MissingField(&'static str),

    #[error("invalid value in field '{field}': {msg}")]
    InvalidValue { field: &'static str, msg: String },
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let xml = fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        parse_xml(&xml)
    }
}

fn parse_xml(xml: &str) -> Result<AppConfig, ConfigError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    let mut package_name: Option<String> = None;
    let mut repo_source: Option<String> = None;
    let mut mode: Option<String> = None;
    let mut ascii_tree: Option<String> = None;
    let mut exclude_filter: Option<String> = None;
    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_tag = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
            }
            Ok(Event::Text(e)) => {
                if let Some(tag) = &current_tag {
                    let value = unescape(std::str::from_utf8(e.as_ref())
                        .map_err(|err| ConfigError::XmlError(err.to_string()))?)
                        .map_err(|err| ConfigError::XmlError(err.to_string()))?
                        .into_owned();


                    match tag.as_str() {
                        "PackageName" => package_name = Some(value),
                        "RepoSource" => repo_source = Some(value),
                        "Mode" => mode = Some(value),
                        "AsciiTree" => ascii_tree = Some(value),
                        "ExcludeFilter" => exclude_filter = Some(value),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => current_tag = None,
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConfigError::XmlError(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    let package_name = package_name.ok_or(ConfigError::MissingField("PackageName"))?;
    if package_name.trim().is_empty() {
        return Err(ConfigError::MissingField("PackageName"));
    }

    let repo_source = repo_source.ok_or(ConfigError::MissingField("RepoSource"))?;
    if repo_source.trim().is_empty() {
        return Err(ConfigError::MissingField("RepoSource"));
    }

    let mode = mode.ok_or(ConfigError::MissingField("Mode"))?;
    let mode_trim = mode.trim();
    if mode_trim != "real" && mode_trim != "test" {
        return Err(ConfigError::InvalidValue {
            field: "Mode",
            msg: "expected 'real' or 'test'".into(),
        });
    }

    let ascii_tree_raw = ascii_tree.ok_or(ConfigError::MissingField("AsciiTree"))?;
    let ascii_tree_bool = match ascii_tree_raw.trim() {
        "true" | "True" | "TRUE" => true,
        "false" | "False" | "FALSE" => false,
        other => {
            return Err(ConfigError::InvalidValue {
                field: "AsciiTree",
                msg: format!("expected true/false, got '{other}'"),
            })
        }
    };

    let exclude_filter = exclude_filter.unwrap_or_default();

    Ok(AppConfig {
        package_name,
        repo_source,
        mode: mode_trim.to_string(),
        ascii_tree: ascii_tree_bool,
        exclude_filter,
    })
}
