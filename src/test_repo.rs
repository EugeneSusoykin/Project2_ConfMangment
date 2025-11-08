// Модуль для загрузки тестового репозитория зависимостей из файла

use std::collections::HashMap;
use std::fs;


/// Возможные ошибки при работе с тестовым репозиторием
#[derive(Debug)]
pub enum TestRepoError {
    ReadError(String),
    ParseError(String),
}

/// Загрузка тестового репозитория из текстового файла.
pub fn load_test_repo(path: &str) -> Result<HashMap<String, Vec<String>>, TestRepoError> {
    // Считывание файла в строку
    let content = fs::read_to_string(path)
        .map_err(|_| TestRepoError::ReadError(format!("Cannot read test repository file: {}", path)))?;

    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // пропускаем пустые строки
        }

        // Разделение строки на имя пакета и список зависимостей
        if let Some((pkg, deps)) = trimmed.split_once(':') {
            let pkg = pkg.trim();
            let deps: Vec<String> = deps
                .split_whitespace()
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();

            map.insert(pkg.to_string(), deps);
        } else {
            return Err(TestRepoError::ParseError(format!(
                "Invalid line format: '{}'",
                trimmed
            )));
        }
    }

    Ok(map)
}
