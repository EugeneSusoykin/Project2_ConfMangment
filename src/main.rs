// Главный модуль для анализа зависимостей Rust-пакета
// Запуск:
//   cargo run -- <config.xml - обычный вывод дерева зависимостей
//   cargo run -- <config.xml> --reverse - вывод дерева обратных зависимостей

mod config;
mod cargo_parser;
mod graph;
mod test_repo;

use std::env;

use config::{AppConfig, ConfigError};
use cargo_parser::get_dependencies;
use graph::DependencyGraph;
use test_repo::load_test_repo;
use crate::cargo_parser::CargoParseError;


/// Точка входа в приложение: загружает конфигурацию и извлекает зависимости пакета
fn main() {
    // Чтение аргументов командной строки
    let mut args = env::args().skip(1);
    let config_path = args.next().unwrap_or_else(|| "config.example.xml".to_string());
    let reverse = args.next().map(|s| s == "--reverse").unwrap_or(false);

    // Загружаем конфиг
    let cfg = match AppConfig::load_from_file(&config_path) {
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

    if cfg.mode == "test" {
        // Режим тестового репозитория (Этапы 2–4)
        println!("\nRunning in TEST mode (text file parsing)");
        match load_test_repo(&cfg.repo_source) {
            Ok(map) => {
                // Загружаем полный граф из файла
                graph.load_from_map(&map);

                if reverse {
                    println!("\nReverse dependencies for '{}' ", cfg.package_name);
                    graph.print_reverse_tree(&cfg.package_name, &cfg.exclude_filter);
                } else {
                    println!("\nDependencies for '{}' ", cfg.package_name);
                    graph.print_tree(&cfg.package_name, &cfg.exclude_filter);
                }
            }
            Err(e) => {
                print_test_repo_error(e);
                return;
            }
        }
    } else {
        // Нормальный режим: прямые зависимости корневого пакета
        println!("\nRunning in NORMAL mode (Cargo.toml)");
        match get_dependencies(&cfg.repo_source) {
            Ok(deps) => {
                // Создание небольшого графа: корень -> каждая зависимость
                for d in deps {
                    graph.add_edge(&cfg.package_name, &d);
                }

                if reverse {
                    println!("\nNOTE: reverse mode is only meaningful with a full graph.");
                    println!("Showing who depends on '{}' among the known nodes only:\n", cfg.package_name);
                    graph.print_reverse_tree(&cfg.package_name, &cfg.exclude_filter);
                } else if cfg.ascii_tree {
                    println!("\nDependencies for '{}' ", cfg.package_name);
                    graph.print_tree(&cfg.package_name, &cfg.exclude_filter);
                } else {
                    println!("\nDirect package dependencies '{}':", cfg.package_name);
                    for dep in graph.nodes.get(&cfg.package_name).map(|n| &n.dependencies).unwrap_or(&Vec::new()) {
                        println!("- {}", dep);
                    }
                }
            }
            Err(e) => {
                print_cargo_error(e);
                return;
            }
        }
    }
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
            eprintln!("CARGO ERROR: cannot read Cargo.toml: {}", msg);
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