mod s0_main_spec;
mod s1_parse_impl;
mod s1_parse_spec;
mod s2_normalize_impl;
mod s2_normalize_spec;
mod s3_graph_impl;
mod s3_graph_spec;
mod s4_order_impl;
mod s4_order_spec;
mod s5_symbol_impl;
mod s5_symbol_spec;
mod s6_mmap_impl;
mod s6_mmap_spec;
mod s7_reloc_impl;
mod s7_reloc_spec;
mod s8_finalize_impl;
mod s8_finalize_spec;
mod s9_runtime;

use vstd::prelude::*;

verus! {

#[verifier::external_body]
pub fn print_debug<T: std::fmt::Debug>(name: &str, value: &T) {
    println!("{name}={value:#?}");
}

pub fn run_planner(input: s0_main_spec::PlannerInput, debug: bool) -> (out: Result<s8_finalize_spec::RuntimePlan, String>)
    ensures
        match out {
            Ok(o) => s0_main_spec::spec(input, o),
            Err(_) => true,
        },
{
    let parsed_objects = s1_parse_impl::run(input)?;
    if debug {
        print_debug("parsed_objects", &parsed_objects);
    }

    let normalized_objects = s2_normalize_impl::run(&parsed_objects)?;
    if debug {
        print_debug("normalized_objects", &normalized_objects);
    }

    let dependency_graph = s3_graph_impl::run(&normalized_objects)?;
    if debug {
        print_debug("dependency_graph", &dependency_graph);
    }

    let dependency_order = s4_order_impl::run(&normalized_objects, &dependency_graph)?;
    if debug {
        print_debug("dependency_order", &dependency_order);
    }

    let symbol_bindings = s5_symbol_impl::run(&normalized_objects, &dependency_order)?;
    if debug {
        print_debug("symbol_bindings", &symbol_bindings);
    }

    let memory_map_plan = s6_mmap_impl::run(&normalized_objects, &dependency_order)?;
    if debug {
        print_debug("memory_map_plan", &memory_map_plan);
    }

    let relocation_writes = s7_reloc_impl::run(
        &normalized_objects,
        &dependency_order,
        &symbol_bindings,
        &memory_map_plan,
    )?;
    if debug {
        print_debug("relocation_writes", &relocation_writes);
    }

    let runtime_plan = s8_finalize_impl::run(
        &normalized_objects,
        &dependency_order,
        &memory_map_plan,
        &relocation_writes,
    )?;

    if debug {
        print_debug("runtime_plan", &runtime_plan);
    }
    Ok(runtime_plan)
}

}

pub struct CliArgs {
    // veriload [--debug] <path> [<path> ...] [-- <args> ...]
    pub debug: bool,
    pub paths: Vec<String>,
    pub args: Vec<String>,
}

fn parse_cli_args() -> Result<CliArgs, String> {
    Err("TODO: parse cli args".to_string())
}

fn cli_args_to_planner_input(_cli: &CliArgs) -> Result<s0_main_spec::PlannerInput, String> {
    Err("TODO: cli -> planner input".to_string())
}

fn run() -> Result<(), String> {
    let cli = parse_cli_args()?;
    let planner_input = cli_args_to_planner_input(&cli)?;
    let runtime_plan = run_planner(planner_input, cli.debug)?;
    s9_runtime::run_runtime(runtime_plan, cli.args)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
