mod debug;
mod discover_impl;
mod discover_spec;
mod consts;
mod final_stage_impl;
mod final_stage_spec;
mod main_spec;
mod mmap_plan_impl;
mod mmap_plan_spec;
mod parse_impl;
mod parse_spec;
mod relocate_plan_impl;
mod relocate_apply_impl;
mod relocate_apply_spec;
mod relocate_plan_spec;
mod runtime;
mod resolve_impl;
mod resolve_spec;
mod types;

use crate::debug::print_loader_plan;
use crate::types::{LoaderError, LoaderInput, LoaderObject, LoaderOutput};
use vstd::prelude::*;

verus! {

pub fn plan_loader(input: LoaderInput) -> (out: Result<LoaderOutput, LoaderError>)
    ensures
        main_spec::plan_result_spec(input, out),
{
    let parsed_res = parse_impl::parse_stage(input);
    match parsed_res {
        Err(e) => Err(e),
        Ok(parsed) => {
            let discovered_res = discover_impl::discover_stage(&parsed);
            match discovered_res {
                Err(e) => Err(e),
                Ok(discovered) => {
                    let resolved_res = resolve_impl::resolve_stage_ref(&parsed, &discovered);
                    match resolved_res {
                        Err(e) => Err(e),
                        Ok(resolved) => {
                            let mmap_plans_res = mmap_plan_impl::mmap_plan_stage(&parsed, &discovered);
                            match mmap_plans_res {
                                Err(e) => Err(e),
                                Ok(mmap_plans) => {
                                    let plan_reloc_res = relocate_plan_impl::plan_relocate_stage(
                                        parsed,
                                        discovered,
                                        resolved,
                                        mmap_plans,
                                    );
                                    match plan_reloc_res {
                                        Err(e) => Err(e),
                                        Ok(plan_reloc) => {
                                            let reloc_apply_res = relocate_apply_impl::relocate_apply_stage(plan_reloc);
                                            match reloc_apply_res {
                                                Err(e) => Err(e),
                                                Ok(reloc_applied) => final_stage_impl::final_stage(reloc_applied),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

} // verus!

fn read_loader_input(paths: &[String]) -> Result<LoaderInput, LoaderError> {
    let mut objects: Vec<LoaderObject> = Vec::new();
    for path in paths {
        let bytes = match std::fs::read(path) {
            Ok(v) => v,
            Err(_) => {
                return Err(LoaderError {});
            }
        };

        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());

        objects.push(LoaderObject { name, bytes });
    }

    Ok(LoaderInput { objects })
}

fn run_program(paths: &[String], print_debug: bool) {
    let input = match read_loader_input(paths) {
        Ok(v) => v,
        Err(_) => panic!("planning failed"),
    };

    let plan = match plan_loader(input) {
        Ok(v) => v,
        Err(_) => panic!("planning failed"),
    };

    if print_debug {
        print_loader_plan(&plan);
    }

    if runtime::run_runtime(&plan).is_err() {
        panic!("main failed");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (print_debug, paths): (bool, &[String]) = if args.len() >= 2 && args[1] == "--debug" {
        (true, &args[2..])
    } else {
        (false, &args[1..])
    };

    if paths.is_empty() {
        eprintln!("usage:");
        eprintln!("  veriload [--debug] <elf> [<elf> ...]");
        return;
    }

    run_program(paths, print_debug);
}
