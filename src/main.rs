// Главный модуль для анализа зависимостей Rust-пакета

mod config;
mod cargo_parser;
mod graph;
mod test_repo;

use config::{AppConfig, ConfigError};

use std::env;
use cargo_parser::get_dependencies;
use graph::DependencyGraph;
use test_repo::load_test_repo;
use std::collections::HashSet;
use crate::cargo_parser::CargoParseError;


/// Точка входа в приложение: загружает конфигурацию и извлекает зависимости пакета
fn main() {
    // Путь до XML из аргументов
    // cargo run -- ./config.example.xml
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.xml"
    };

    // Загружаем конфиг
    let cfg = match AppConfig::load_from_file(config_path) {
        Ok(c) => c,
        Err(e) => {
            print_config_error(e);
            return;
        }
    };

    println!("Config was uploaded successfully");
    println!("{:#?}", cfg);

    // Создание графа зависимостей
    let mut graph = DependencyGraph::new();

    // Режим тестирования (Тестовый репозиторий)
    if cfg.mode == "test" {
        println!("\nRunning in TEST mode (test_repo.txt)");
        match load_test_repo(&cfg.repo_source) {
            Ok(repo_data) => {
                for (pkg, deps) in repo_data {
                    graph.add_package(&pkg, deps);
                }
            }
            Err(e) => { print_test_repo_error(e); return; }
        }
    } else {
        // Режим нормального анализа (Настоящий репозиторий)
        println!("\nRunning in NORMAL mode (real dependency parsing)");
        match get_dependencies(&cfg.repo_source) {
            Ok(deps) => {
                graph.add_package(&cfg.package_name, deps);
            }
            Err(e) => {
                print_cargo_error(e);
                return;
            }
        }
    }

    println!("\nDependency graph traversal (DFS):\n");

    let mut visited = HashSet::new();
    let mut path = Vec::new();

    graph.dfs(&cfg.package_name, &cfg.exclude_filter, &mut visited, &mut path);

}

/// Обработчик ошибок конфигурационного файла (config.xml)
fn print_config_error(err: ConfigError) {
    match err {
        ConfigError::ReadError(msg) => {
            eprintln!("CONFIG ERROR: cannot read file: {}", msg);
        }
        ConfigError::XmlError(msg) => {
            eprintln!("CONFIG ERROR: invalid XML: {}", msg);
        }
        ConfigError::MissingField(field) => {
            eprintln!("CONFIG ERROR: missing required field '{}'", field);
        }
        ConfigError::InvalidValue { field, msg } => {
            eprintln!("CONFIG ERROR: invalid value in '{}': {}", field, msg);
        }
    }
}

/// Обработчик ошибок при работе с локальным или удалённым Cargo.toml
fn print_cargo_error(err: CargoParseError) {
    match err {
        CargoParseError::NetworkError(msg) => {
            eprintln!("CARGO ERROR: cannot fetch Cargo.toml: {}", msg);
        }
        CargoParseError::FileError(msg) => {
            eprintln!("CARGO ERROR: cannot read local Cargo.toml: {}", msg);
        }
        CargoParseError::ParseError => {
            eprintln!("CARGO ERROR: invalid Cargo.toml format");
        }
    }
}

/// Обработчик ошибок при работе с тестовым репозиторием
fn print_test_repo_error(err: test_repo::TestRepoError) {
    match err {
        test_repo::TestRepoError::ReadError(msg) => {
            eprintln!("TEST REPO ERROR: {}", msg);
        }
        test_repo::TestRepoError::ParseError(msg) => {
            eprintln!("TEST REPO ERROR: invalid file format: {}", msg);
        }
    }
}