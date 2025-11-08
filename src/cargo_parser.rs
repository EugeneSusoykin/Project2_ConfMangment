// Модуль для получения и разбора локального или удалённого Cargo.toml пакета

use std::fs;
use std::path::Path;

use reqwest::blocking;
use thiserror::Error;


/// Перечисление возможных ошибок при работе с Cargo.toml
#[derive(Debug, Error)]
pub enum CargoParseError {
    #[error("failed to fetch Cargo.toml: {0}")]
    NetworkError(String),

    #[error("failed to read local Cargo.toml: {0}")]
    FileError(String),

    #[error("invalid Cargo.toml format")]
    ParseError,
}

/// Извлекает список прямых зависимостей из Cargo.toml
pub fn get_dependencies(repo_source: &str) -> Result<Vec<String>, CargoParseError> {
    let content = if repo_source.starts_with("http") {
        println!("Загрузка Cargo.toml из репозитория: {}", repo_source);

        // Преобразование URL репозитория в ссылку на сырой Cargo.toml
        // https://github.com/tokio-rs/tokio ->
        // https://raw.githubusercontent.com/tokio-rs/tokio/master/Cargo.toml
        let cargo_url = if repo_source.ends_with(".git") {
            format!("{}/master/Cargo.toml", repo_source.trim_end_matches(".git"))
        } else {
            format!(
                "https://raw.githubusercontent.com/{}/master/Cargo.toml",
                repo_source
                    .trim_start_matches("https://github.com/")
                    .trim_end_matches('/')
            )
        };

        blocking::get(&cargo_url)
            .map_err(|e| CargoParseError::NetworkError(e.to_string()))?
            .text()
            .map_err(|e| CargoParseError::NetworkError(e.to_string()))?
    } else if Path::new(repo_source).exists() {
        println!("Чтение локального Cargo.toml: {}", repo_source);
        fs::read_to_string(repo_source)
            .map_err(|e| CargoParseError::FileError(e.to_string()))?
    } else {
        return Err(CargoParseError::FileError(format!(
            "Файл или URL не найден: {}",
            repo_source
        )));
    };

    // Извлечение секции [dependencies]
    let deps_section = extract_dependencies_section(&content)?;
    let deps = parse_dependencies(&deps_section);
    Ok(deps)
}

// Поиск и возврат секции [dependencies]
fn extract_dependencies_section(cargo_toml: &str) -> Result<String, CargoParseError> {
    let mut lines = Vec::new();
    let mut in_deps = false;

    for line in cargo_toml.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && in_deps && trimmed != "[dependencies]" {
            // выход из секции зависимостей
            break;
        }

        if in_deps {
            lines.push(trimmed.to_string());
        }

        if trimmed == "[dependencies]" {
            in_deps = true;
        }
    }

    if lines.is_empty() {
        return Err(CargoParseError::ParseError);
    }

    Ok(lines.join("\n"))
}

/// Разбор зависимостей в формате `name = "version"`
fn parse_dependencies(section: &str) -> Vec<String> {
    let mut deps = Vec::new();

    for line in section.lines() {
        if let Some((name, _)) = line.split_once('=') {
            let dep_name = name.trim();
            if !dep_name.is_empty() {
                deps.push(dep_name.to_string());
            }
        }
    }

    deps
}
