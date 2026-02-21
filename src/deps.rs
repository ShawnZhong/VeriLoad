use std::collections::{HashMap, VecDeque};
use std::path::Path;

use crate::dynamic;
use crate::image;
use crate::model::{DynamicInfo, Module};
use crate::parse;
use crate::plan;
use crate::rt;

const SEARCH_PATHS: [&str; 3] = ["/lib", "/usr/lib", "/lib64"];

pub fn normalize_path(path: &str) -> String {
    std::fs::canonicalize(path)
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| path.to_string())
}

pub fn resolve_needed_path(name: &str) -> Option<String> {
    for dir in SEARCH_PATHS {
        let candidate = format!("{dir}/{name}");
        if Path::new(&candidate).is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn load_dependency_graph(main_path: &str) -> Vec<Module> {
    let main_resolved = normalize_path(main_path);
    let main = load_one_module(&main_resolved);

    let mut modules = vec![main];
    let mut by_path: HashMap<String, usize> = HashMap::new();
    by_path.insert(main_resolved.clone(), 0);

    let mut queue: VecDeque<String> = VecDeque::new();
    for needed in modules[0].needed.clone() {
        queue.push_back(needed);
    }

    while let Some(name) = queue.pop_front() {
        let resolved = resolve_needed_path(&name)
            .unwrap_or_else(|| rt::fatal(format!("missing DT_NEEDED library: {name}")));
        let normalized = normalize_path(&resolved);

        if by_path.contains_key(&normalized) {
            continue;
        }

        rt::log(format!("load DSO {normalized}"));
        let module = load_one_module(&normalized);
        let new_index = modules.len();
        by_path.insert(normalized, new_index);

        for needed in module.needed.clone() {
            queue.push_back(needed);
        }

        modules.push(module);
    }

    for i in 0..modules.len() {
        let mut dep_indices: Vec<usize> = Vec::new();
        let needed_list = modules[i].needed.clone();

        for name in needed_list {
            let resolved = resolve_needed_path(&name)
                .unwrap_or_else(|| rt::fatal(format!("missing DT_NEEDED library: {name}")));
            let normalized = normalize_path(&resolved);
            let dep = by_path.get(&normalized).copied().unwrap_or_else(|| {
                rt::fatal(format!(
                    "dependency resolved but not loaded: module={} dep={} path={}",
                    modules[i].path, name, normalized
                ))
            });
            if !dep_indices.contains(&dep) {
                dep_indices.push(dep);
            }
        }

        modules[i].needed_indices = dep_indices;
    }

    modules
}

fn load_one_module(path: &str) -> Module {
    rt::log(format!("parse {path}"));

    let bytes = rt::rt_read_file(path);
    let ehdr = parse::parse_ehdr(&bytes);
    let phdrs = parse::parse_phdrs(&bytes, &ehdr);
    let plan = plan::build_load_plan(bytes.len(), &ehdr, &phdrs);
    let image = image::materialize_image(&bytes, &plan);
    let base = (image as u64)
        .checked_sub(plan.min_vaddr_page)
        .unwrap_or_else(|| rt::fatal(format!("base underflow while loading {path}")));

    let mut module = Module {
        path: normalize_path(path),
        plan,
        image,
        base,
        dynamic: DynamicInfo::default(),
        needed: Vec::new(),
        needed_indices: Vec::new(),
        soname: None,
    };

    let parsed = dynamic::parse_dynamic(&module);
    module.dynamic = parsed.info;
    module.needed = parsed.needed;
    module.soname = parsed.soname;

    module
}
