use verus_builtin::*;
use verus_builtin_macros::*;

mod arith_lemmas;
mod deps;
mod dynamic;
mod image;
mod init;
mod model;
mod parse;
mod plan;
mod protect;
mod relocate;
mod rt;
mod spec;
mod stack;
mod symbols;

fn host_main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        rt::fatal(format!("usage: {} <et_dyn_executable>", args[0]));
    }

    let main_path = deps::normalize_path(&args[1]);
    rt::log(format!("main={main_path}"));

    let modules = deps::load_dependency_graph(&main_path);
    if modules.is_empty() {
        rt::fatal("internal error: module graph is empty");
    }

    let dep_order = init::dependency_first_order(&modules);
    relocate::relocate_all(&modules, &dep_order);
    protect::apply_all_protections(&modules);
    init::run_initializers(&modules, &dep_order);

    let main_module = &modules[0];
    if !spec::executable_va(&main_module.plan, main_module.plan.entry) {
        rt::fatal(format!(
            "entry VA is not executable: module={} entry=0x{:x}",
            main_module.path, main_module.plan.entry
        ));
    }

    let entry = main_module
        .base
        .checked_add(main_module.plan.entry)
        .unwrap_or_else(|| {
            rt::fatal(format!(
                "entry address overflow: module={} base=0x{:x} entry=0x{:x}",
                main_module.path, main_module.base, main_module.plan.entry
            ))
        });

    let sp = stack::build_minimal_stack(&main_path);
    rt::log(format!("enter entry=0x{entry:x} sp=0x{:x}", sp as usize));

    unsafe {
        rt::enter(entry, sp);
    }
}

verus! {

#[verifier::external_body]
fn main() {
    crate::host_main();
}

} // verus!
