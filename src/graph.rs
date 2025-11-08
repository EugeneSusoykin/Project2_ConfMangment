// Модуль для построения и обхода графа зависимостей

use std::collections::{HashMap, HashSet};

/// Узел графа, представляющий отдельный пакет и его зависимости
#[derive(Debug, Clone)]
pub struct PackageNode {
    pub name: String,
    pub dependencies: Vec<String>,
}

/// Структура графа зависимостей
#[derive(Debug)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, PackageNode>,
}

/// Блок реализации структуры DependencyGraph
impl DependencyGraph {
    // Создание пустого графа зависимостей
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    // Добавление пакета и его зависимостей в граф
    pub fn add_package(&mut self, name: &str, deps: Vec<String>) {
        let node = PackageNode {
            name: name.to_string(),
            dependencies: deps,
        };
        self.nodes.insert(name.to_string(), node);
    }

    /// Рекурсивный обход графа зависимостей (DFS)
    pub fn dfs(
        &self,
        package: &str,
        exclude_filter: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) {
        // Проверка на цикл (наличие пакета в текущем пути)
        if stack.contains(&package.to_string()) {
            println!("Cycle detected: {:?}", stack);
            return;
        }

        // Фильтр по подстроке
        if !exclude_filter.is_empty() && package.contains(exclude_filter) {
            return;
        }

        println!("{}", "  ".repeat(stack.len()) + package);
        stack.push(package.to_string());
        visited.insert(package.to_string());

        // Рекурсивный обход зависимостей проекта
        if let Some(node) = self.nodes.get(package) {
            for dep in &node.dependencies {
                self.dfs(dep, exclude_filter, visited, stack);
            }
        }

        stack.pop();
    }
}
