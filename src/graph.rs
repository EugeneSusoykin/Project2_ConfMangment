// Модуль для построения и обхода графа зависимостей

use std::collections::{HashMap, HashSet};

/// Узел графа, представляющий отдельный пакет и его зависимости
#[allow(dead_code)]
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

    /// Проверка присутствия пакета в графе
    pub fn ensure_node(&mut self, name: &str) {
        self.nodes
            .entry(name.to_string())
            .or_insert_with(|| PackageNode { name: name.to_string(), dependencies: Vec::new() });
    }

    /// Добавление ориентированного ребра package -> depends_on
    pub fn add_edge(&mut self, package: &str, depends_on: &str) {
        self.ensure_node(package);
        self.ensure_node(depends_on);
        if let Some(node) = self.nodes.get_mut(package) {
            if !node.dependencies.contains(&depends_on.to_string()) {
                node.dependencies.push(depends_on.to_string());
            }
        }
    }

    /// Загрузка полного графа из "карты тестового репозитория"
    pub fn load_from_map(&mut self, map: &HashMap<String, Vec<String>>) {
        for (pkg, deps) in map {
            self.ensure_node(pkg);
            for d in deps {
                self.add_edge(pkg, d);
            }
        }
    }

    /// Построение обратного отображения
    pub fn build_reverse_index(&self) -> HashMap<String, Vec<String>> {
        let mut rev: HashMap<String, Vec<String>> = HashMap::new();
        for (pkg, node) in &self.nodes {
            for dep in &node.dependencies {
                rev.entry(dep.clone()).or_default().push(pkg.clone());
            }
        }
        rev
    }

    /// Вывод дерева прямых зависимостей с фильтром по подстроке
    pub fn print_tree(&self, root: &str, exclude_filter: &str) {
        let mut visited = HashSet::new();
        let mut stack: Vec<String> = Vec::new();
        self.dfs_forward(root, exclude_filter, &mut visited, &mut stack);
    }

    fn dfs_forward(
        &self,
        package: &str,
        exclude_filter: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) {
        if let Some(f) = exclude_nonempty(exclude_filter) {
            if package.contains(f) {
                return;
            }
        }

        // Проверка бесконечных циклов
        if stack.contains(&package.to_string()) {
            println!("{}{} (cycle)", "  ".repeat(stack.len()), package);
            return;
        }
        if visited.contains(package) {
            println!("{}{} (visited)", "  ".repeat(stack.len()), package);
            return;
        }

        println!("{}{}", "  ".repeat(stack.len()), package);
        visited.insert(package.to_string());
        stack.push(package.to_string());

        if let Some(node) = self.nodes.get(package) {
            for dep in &node.dependencies {
                self.dfs_forward(dep, exclude_filter, visited, stack);
            }
        }

        stack.pop();
    }

    /// Вывод обратных зависимостей для `target`
    pub fn print_reverse_tree(&self, target: &str, exclude_filter: &str) {
        let rev = self.build_reverse_index();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        self.dfs_reverse(target, exclude_filter, &rev, &mut visited, &mut stack);
    }

    fn dfs_reverse(
        &self,
        package: &str,
        exclude_filter: &str,
        rev: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) {
        if let Some(f) = exclude_nonempty(exclude_filter) {
            if package.contains(f) {
                return;
            }
        }

        if stack.contains(&package.to_string()) {
            println!("{}{} (cycle)", "  ".repeat(stack.len()), package);
            return;
        }
        if visited.contains(package) {
            println!("{}{} (visited)", "  ".repeat(stack.len()), package);
            return;
        }

        println!("{}{}", "  ".repeat(stack.len()), package);
        visited.insert(package.to_string());
        stack.push(package.to_string());

        if let Some(parents) = rev.get(package) {
            for p in parents {
                self.dfs_reverse(p, exclude_filter, rev, visited, stack);
            }
        }

        stack.pop();
    }
}

/// Возврат Some(filter) если строка непустая
fn exclude_nonempty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t) }
}
