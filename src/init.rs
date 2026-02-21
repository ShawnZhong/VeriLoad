use crate::image;
use crate::model::Module;
use crate::rt;
use crate::spec;

pub fn dependency_first_order(modules: &[Module]) -> Vec<usize> {
    if modules.is_empty() {
        return Vec::new();
    }

    let mut order: Vec<usize> = Vec::with_capacity(modules.len());
    let mut state: Vec<u8> = vec![0u8; modules.len()];

    dfs_postorder(0, modules, &mut state, &mut order);

    for i in 0..modules.len() {
        if state[i] == 0 {
            dfs_postorder(i, modules, &mut state, &mut order);
        }
    }

    order
}

fn dfs_postorder(idx: usize, modules: &[Module], state: &mut [u8], out: &mut Vec<usize>) {
    match state[idx] {
        2 => return,
        1 => {
            // Cycle: stop descending here, continue with current traversal.
            return;
        }
        _ => {}
    }

    state[idx] = 1;
    for &dep in &modules[idx].needed_indices {
        if dep >= modules.len() {
            rt::fatal(format!(
                "dependency index out of range: module={} dep={} total={}",
                modules[idx].path,
                dep,
                modules.len()
            ));
        }
        dfs_postorder(dep, modules, state, out);
    }
    state[idx] = 2;
    out.push(idx);
}

pub fn run_initializers(modules: &[Module], dep_order: &[usize]) {
    for &idx in dep_order {
        let module = &modules[idx];

        if let Some(init_va) = module.dynamic.init {
            if init_va != 0 {
                call_relative_init(module, init_va, "DT_INIT");
            }
        }

        if let Some(arr) = &module.dynamic.init_array {
            if arr.size % 8 != 0 {
                rt::fatal(format!(
                    "init array size is not a multiple of 8: module={} size=0x{:x}",
                    module.path, arr.size
                ));
            }

            let count = (arr.size / 8) as usize;
            for i in 0..count {
                let slot_va = arr
                    .addr
                    .checked_add((i as u64) * 8)
                    .unwrap_or_else(|| rt::fatal("init array slot overflow"));
                let fn_addr = image::read_u64(module, slot_va);
                if fn_addr == 0 {
                    continue;
                }

                if !is_executable_address(modules, fn_addr) {
                    rt::fatal(format!(
                        "init_array pointer is not executable mapped address: module={} addr=0x{:x}",
                        module.path, fn_addr
                    ));
                }

                call_absolute_init(fn_addr, &format!("DT_INIT_ARRAY[{}]", i), &module.path);
            }
        }
    }
}

fn call_relative_init(module: &Module, init_va: u64, tag: &str) {
    if !spec::executable_va(&module.plan, init_va) {
        rt::fatal(format!(
            "{} target is not executable: module={} va=0x{:x}",
            tag, module.path, init_va
        ));
    }
    let addr = module
        .base
        .checked_add(init_va)
        .unwrap_or_else(|| rt::fatal("init address overflow"));
    call_absolute_init(addr, tag, &module.path);
}

fn call_absolute_init(addr: u64, tag: &str, module_path: &str) {
    rt::log(format!("call {tag} module={module_path} addr=0x{addr:x}"));
    let f: extern "C" fn() = unsafe { std::mem::transmute(addr as usize) };
    f();
}

fn is_executable_address(modules: &[Module], absolute_addr: u64) -> bool {
    for module in modules {
        if absolute_addr < module.base {
            continue;
        }
        let rel = absolute_addr - module.base;
        if spec::executable_va(&module.plan, rel) {
            return true;
        }
    }
    false
}
