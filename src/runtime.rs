use std::env;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::planner::derive_stage_state;
use crate::rt_common::{fatal, log, LoaderOutput, RuntimePlan};

fn run_unverified_runtime(output: &LoaderOutput, target: &Path, passthrough: &[OsString]) -> ! {
    // Runtime handoff is intentionally unverified: we delegate execution to the kernel/OS loader.
    log(&format!(
        "plan entry=0x{:x} init_calls={} maps={}",
        output.entry_pc,
        output.initializers.len(),
        output.mmap_plans.len()
    ));
    let err = Command::new(target).args(passthrough).exec();
    fatal(&format!(
        "exec failed for {}: {}",
        target.to_string_lossy(),
        err
    ));
}

pub fn load_and_plan_unverified_impl() -> RuntimePlan {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 2 {
        fatal("usage: ./veriload <et_dyn_executable> [args...]");
    }
    let main_path = PathBuf::from(&args[1]);
    log(&format!("main={}", main_path.to_string_lossy()));
    let stage = derive_stage_state(&main_path);
    RuntimePlan {
        target: main_path,
        passthrough: args[2..].to_vec(),
        stage,
    }
}

pub fn runtime_handoff_unverified_impl(plan: RuntimePlan) -> ! {
    run_unverified_runtime(&plan.stage.output, &plan.target, &plan.passthrough)
}

pub fn run_impl() -> ! {
    let plan = load_and_plan_unverified_impl();
    runtime_handoff_unverified_impl(plan);
}
