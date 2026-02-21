use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::elf::parse_elf;
use crate::rt_common::*;

fn split_search_paths(raw: &str) -> Vec<String> {
    raw.replace(';', ":")
        .split(':')
        .filter(|p| !p.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn resolve_needed_path(requester: &ParsedObject, name: &str) -> PathBuf {
    if name.contains('/') {
        let p = PathBuf::from(name);
        if p.exists() {
            return canonical_or(&p);
        }
        fatal(&format!("missing needed object: requester={} needed={}", requester.name, name));
    }

    let mut dirs: Vec<String> = Vec::new();
    if let Some(rpath) = &requester.rpath {
        dirs.extend(split_search_paths(rpath));
    }
    if let Ok(ld_path) = env::var("LD_LIBRARY_PATH") {
        dirs.extend(split_search_paths(&ld_path));
    }
    dirs.push("/lib".to_string());
    dirs.push("/usr/lib".to_string());

    for dir in dirs {
        let candidate = Path::new(&dir).join(name);
        if candidate.exists() {
            return canonical_or(&candidate);
        }
    }
    fatal(&format!("missing needed object: requester={} needed={}", requester.name, name));
}

fn load_object_from_path(id: usize, path: &Path) -> ParsedObject {
    let display = path_display(path);
    log(&format!("parse {}", display));
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => fatal(&format!("failed to read {}: {}", display, e)),
    };
    parse_elf(id, path, bytes)
}

fn compute_bfs_order(edges: &[Vec<usize>], root: usize) -> Vec<usize> {
    let mut order: Vec<usize> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut q: VecDeque<usize> = VecDeque::new();
    q.push_back(root);
    while let Some(id) = q.pop_front() {
        if !seen.insert(id) {
            continue;
        }
        order.push(id);
        for dep in &edges[id] {
            q.push_back(*dep);
        }
    }
    order
}

fn discover_dependencies(root_path: &Path) -> (Vec<ParsedObject>, Vec<Vec<usize>>, Vec<usize>) {
    let root_path = canonical_or(root_path);
    let mut objects: Vec<ParsedObject> = Vec::new();
    let mut edges: Vec<Vec<usize>> = Vec::new();
    let mut by_path: HashMap<PathBuf, usize> = HashMap::new();

    let root = load_object_from_path(0, &root_path);
    by_path.insert(root.path.clone(), 0);
    objects.push(root);
    edges.push(Vec::new());

    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(0);

    while let Some(id) = queue.pop_front() {
        let requester = objects[id].clone();
        let mut dep_ids: Vec<usize> = Vec::new();

        if id == 0 {
            if let Some(interp) = &requester.interp {
                log(&format!("load DSO {}", interp));
                let interp_path = canonical_or(Path::new(interp));
                let dep_id = if let Some(existing) = by_path.get(&interp_path) {
                    *existing
                } else {
                    let new_id = objects.len();
                    let obj = load_object_from_path(new_id, &interp_path);
                    by_path.insert(interp_path.clone(), new_id);
                    objects.push(obj);
                    edges.push(Vec::new());
                    queue.push_back(new_id);
                    new_id
                };
                dep_ids.push(dep_id);
            }
        }

        for needed in &requester.needed {
            let dep_path = resolve_needed_path(&requester, needed);
            let dep_id = if let Some(existing) = by_path.get(&dep_path) {
                *existing
            } else {
                log(&format!("load DSO {}", path_display(&dep_path)));
                let new_id = objects.len();
                let obj = load_object_from_path(new_id, &dep_path);
                by_path.insert(dep_path.clone(), new_id);
                objects.push(obj);
                edges.push(Vec::new());
                queue.push_back(new_id);
                new_id
            };
            dep_ids.push(dep_id);
        }
        edges[id] = dep_ids;
    }

    let bfs_order = compute_bfs_order(&edges, 0);
    (objects, edges, bfs_order)
}

fn build_layout(objects: &[ParsedObject], bfs_order: &[usize]) -> (Vec<u64>, Vec<SegmentMapPlan>) {
    let mut load_bias: Vec<u64> = vec![0; objects.len()];
    let mut next_dyn = DYN_BASE_START;
    for &obj_id in bfs_order {
        let obj = &objects[obj_id];
        if obj.id == 0 {
            load_bias[obj_id] = 0;
            continue;
        }
        let mut low = u64::MAX;
        let mut high = 0u64;
        for ph in &obj.program_headers {
            if ph.p_type != PT_LOAD {
                continue;
            }
            low = low.min(align_down(ph.p_vaddr, PAGE_SIZE));
            high = high.max(align_up(ph.p_vaddr.saturating_add(ph.p_memsz), PAGE_SIZE));
        }
        if low == u64::MAX || high <= low {
            fatal("invalid PT_LOAD range for DSO");
        }
        let span = high - low;
        let bias = next_dyn.saturating_sub(low);
        load_bias[obj_id] = bias;
        next_dyn = next_dyn.saturating_add(span).saturating_add(PAGE_SIZE);
    }

    let mut plans: Vec<SegmentMapPlan> = Vec::new();
    for &obj_id in bfs_order {
        let obj = &objects[obj_id];
        for (ph_index, ph) in obj.program_headers.iter().enumerate() {
            if ph.p_type != PT_LOAD {
                continue;
            }
            let start = load_bias[obj_id].saturating_add(ph.p_vaddr);
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let off = ph.p_offset as usize;
            if off + filesz > obj.bytes.len() {
                fatal("PT_LOAD file bytes out of range");
            }
            let mut map_bytes = vec![0u8; memsz];
            map_bytes[..filesz].copy_from_slice(&obj.bytes[off..off + filesz]);
            let prot = SegmentPerm {
                read: (ph.p_flags & PF_R) != 0,
                write: (ph.p_flags & PF_W) != 0,
                execute: (ph.p_flags & PF_X) != 0,
            };
            plans.push(SegmentMapPlan {
                object_id: obj_id,
                ph_index,
                start,
                bytes: map_bytes,
                prot,
            });
        }
    }
    (load_bias, plans)
}

fn build_initializers(objects: &[ParsedObject], edges: &[Vec<usize>], load_bias: &[u64]) -> Vec<InitializerCall> {
    fn dfs(
        id: usize,
        objects: &[ParsedObject],
        edges: &[Vec<usize>],
        load_bias: &[u64],
        visiting: &mut HashSet<usize>,
        visited: &mut HashSet<usize>,
        out: &mut Vec<InitializerCall>,
    ) {
        if visited.contains(&id) {
            return;
        }
        if visiting.contains(&id) {
            // Circular dependency: order among the cycle is unspecified by ELF.
            return;
        }
        visiting.insert(id);
        for dep in &edges[id] {
            dfs(*dep, objects, edges, load_bias, visiting, visited, out);
        }
        visiting.remove(&id);
        visited.insert(id);
        if id != 0 {
            if let Some(init) = objects[id].init {
                out.push(InitializerCall {
                    object_id: id,
                    pc: load_bias[id].saturating_add(init),
                });
            }
        }
    }

    let mut visiting: HashSet<usize> = HashSet::new();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut out: Vec<InitializerCall> = Vec::new();
    dfs(0, objects, edges, load_bias, &mut visiting, &mut visited, &mut out);
    out
}

fn finalize_output(
    objects: &[ParsedObject],
    load_bias: &[u64],
    segment_plans: Vec<SegmentMapPlan>,
    initializers: Vec<InitializerCall>,
) -> LoaderOutput {
    let root = &objects[0];
    LoaderOutput {
        entry_pc: load_bias[0].saturating_add(root.entry),
        initializers,
        mmap_plans: segment_plans,
    }
}

pub fn derive_stage_state(main_path: &Path) -> StageState {
    let (objects, edges, bfs_order) = discover_dependencies(main_path);
    let (load_bias, segment_plans) = build_layout(&objects, &bfs_order);
    let initializers = build_initializers(&objects, &edges, &load_bias);
    let output = finalize_output(&objects, &load_bias, segment_plans.clone(), initializers.clone());
    StageState {
        objects,
        edges,
        bfs_order,
        load_bias,
        segment_plans,
        initializers,
        output,
    }
}
