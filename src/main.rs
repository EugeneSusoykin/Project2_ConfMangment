// Главный модуль для анализа зависимостей Rust-пакета

mod config;
mod cargo_parser;

use config::{AppConfig, ConfigError};

use std::env;
use cargo_parser::get_dependencies;
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

    println!("Getting a package dependencies");

    match get_dependencies(&cfg.repo_source) {
        Ok(deps) => {
            println!("\nDirect package dependencies '{}':", cfg.package_name);
            for dep in deps {
                println!("- {}", dep);
            }
        }
        Err(e) => print_cargo_error(e)
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
            eprintln!("CARGO ERROR: cannot read local Cargo.toml: {}", msg);
        }
        CargoParseError::ParseError => {
            eprintln!("CARGO ERROR: invalid Cargo.toml format");
        }
    }
}
