// Главный модуль для анализа зависимостей Rust-пакета
// Запуск:
//    cargo run -- ./config.example.xml - обычный вывод дерева зависимостей
//    cargo run -- ./config.example.xml --reverse - вывод дерева обратных зависимостей
//    cargo run -- ./config.example.xml --d2 deps.d2 --render deps.png --open - обычный вывод
// дерева зависимостей и d2 диаграммы
//    cargo run -- <config.xml> --reverse --d2 deps.d2 --render deps.png --open - вывод дерева
// обратных зависимостей и d2 диаграммы

mod config;
mod cargo_parser;
mod graph;
mod test_repo;
mod d2;

use std::env;

use config::{AppConfig, ConfigError};
use cargo_parser::get_dependencies;
use graph::DependencyGraph;
use test_repo::load_test_repo;
use std::fs;
use which::which;
use crate::cargo_parser::CargoParseError;


/// Точка входа в приложение: загружает конфигурацию и извлекает зависимости пакета
fn main() {
    // Чтение аргументов командной строки
    let args: Vec<String> = env::args().collect();
    let mut d2_path: Option<String> = None;
    let mut render_path: Option<String> = None;
    let mut open_after_render = false;
    // Парсер флагов
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--d2" => {
                if i + 1 < args.len() {
                    d2_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("ERROR: --d2 requires a file path");
                    return;
                }
            }
            "--render" => {
                if i + 1 < args.len() {
                    render_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("ERROR: --render requires a file path (e.g. output.svg)");
                    return;
                }
            }
            "--open" => {
                open_after_render = true;
            }
            _ => {}
        }
        i += 1;
    }

    let config_path = args.get(1).cloned().unwrap_or_else(|| "config.example.xml".to_string());
    let reverse = args.iter().any(|s| s == "--reverse");

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

    // Экспорт в D2 и рендер изображения
    if d2_path.is_some() || render_path.is_some() {
        let d2_text = d2::to_d2(&graph, reverse);
        if let Some(path) = &d2_path {
            if let Err(e) = fs::write(path, &d2_text) {
                eprintln!("D2 ERROR: cannot write {}: {}", path, e);
                return;
            } else {
                println!("D2 saved to {}", path);
            }
        } else {
            let default_path = "graph.d2";
            if let Err(e) = fs::write(default_path, &d2_text) {
                eprintln!("D2 ERROR: cannot write {}: {}", default_path, e);
                return;
            } else {
                println!("D2 saved to {}", default_path);
                d2_path = Some(default_path.to_string());
            }
        }

        if let Some(out_img) = &render_path {
            match which("d2") {
                Ok(bin) => {
                    let input = d2_path.as_ref().unwrap();
                    println!("Rendering with {:?}: {} -> {}", bin, input, out_img);
                    let status = std::process::Command::new(bin)
                        .arg(input)
                        .arg(out_img)
                        .status();

                    match status {
                        Ok(s) if s.success() => {
                            println!("Rendered image: {}", out_img);
                            if open_after_render {
                                if let Err(e) = open::that(out_img) {
                                    eprintln!("OPEN WARN: cannot open {}: {}", out_img, e);
                                }
                            }
                        }
                        Ok(s) => {
                            eprintln!("D2 RENDER ERROR: exit code {:?}", s.code());
                        }
                        Err(e) => {
                            eprintln!("D2 RENDER ERROR: {}", e);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("D2 RENDER SKIPPED: 'd2' CLI not found in PATH.");
                }
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