mod config;

use config::{AppConfig, ConfigError};
use std::env;
use std::process;

fn main() {
    // 1. Берём путь до XML из аргументов
    //    cargo run -- ./config.example.xml
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config.xml>", args[0]);
        process::exit(1);
    }

    let config_path = &args[1];

    // 2. Пытаемся загрузить конфиг
    let cfg = match AppConfig::load_from_file(config_path) {
        Ok(c) => c,
        Err(e) => {
            print_config_error(e);
            process::exit(1);
        }
    };

    // 3. (Этап 1) Просто вывести параметры в формате ключ=значение
    println!("packageName={}", cfg.package_name);
    println!("repoSource={}", cfg.repo_source);
    println!("mode={}", cfg.mode);
    println!("asciiTree={}", cfg.ascii_tree);
    println!("excludeFilter={}", cfg.exclude_filter);
}

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
