// Модуль для формирования D2-представления графа зависимостей

use crate::graph::DependencyGraph;
use std::collections::HashSet;

pub fn to_d2(graph: &DependencyGraph, reverse: bool) -> String {
    // Сбор множества уникальных рёбер вида "A -> B"
    let mut edges = HashSet::<(String, String)>::new();

    for (name, node) in &graph.nodes {
        for dep in &node.dependencies {
            if reverse {
                edges.insert((dep.clone(), name.clone()));
            } else {
                edges.insert((name.clone(), dep.clone()));
            }
        }
    }

    let mut out = String::new();
    out.push_str("direction: right\n\n");

    // Объявление узлов (если будут висячие вершины без рёбер)
    for name in graph.nodes.keys() {
        out.push_str(&format!("{}: {}\n", sanitize(name), name));
    }
    out.push('\n');

    // Рёбра
    for (a, b) in edges {
        out.push_str(&format!("{} -> {}\n", sanitize(&a), sanitize(&b)));
    }

    out
}

// D2-идентификатор (Оставляет только [A-Za-z0-9_], остальное заменяет на '_')
fn sanitize(s: &str) -> String {
    let mut id = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            id.push(ch);
        } else {
            id.push('_');
        }
    }
    if id.is_empty() { "_".to_string() } else { id }
}
