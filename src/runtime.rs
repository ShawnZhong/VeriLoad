use crate::types::{LoaderError, LoaderOutput, MmapPlan, ProtFlags};
use core::arch::asm;
use std::ffi::c_void;
use std::ptr;

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const PROT_EXEC: i32 = 0x4;

const MAP_PRIVATE: i32 = 0x02;
const MAP_FIXED: i32 = 0x10;
const MAP_ANONYMOUS: i32 = 0x20;
const MAP_STACK: i32 = 0x20000;

const STACK_SIZE: usize = 8 * 1024 * 1024;

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
}

fn prot_bits(p: ProtFlags) -> i32 {
    let mut out = 0;
    if p.read {
        out |= PROT_READ;
    }
    if p.write {
        out |= PROT_WRITE;
    }
    if p.execute {
        out |= PROT_EXEC;
    }
    out
}

fn map_segment(plan: &MmapPlan) -> Result<(), LoaderError> {
    if plan.bytes.is_empty() {
        return Ok(());
    }

    let addr = plan.start as usize as *mut c_void;
    let len = plan.bytes.len();
    let mapped = unsafe {
        mmap(
            addr,
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
            -1,
            0,
        )
    };
    if mapped as isize == -1 {
        return Err(LoaderError {});
    }
    if mapped as usize != plan.start as usize {
        return Err(LoaderError {});
    }

    unsafe {
        ptr::copy_nonoverlapping(plan.bytes.as_ptr(), mapped as *mut u8, len);
    }
    Ok(())
}

fn protect_segment(plan: &MmapPlan) -> Result<(), LoaderError> {
    if plan.bytes.is_empty() {
        return Ok(());
    }

    let addr = plan.start as usize as *mut c_void;
    let len = plan.bytes.len();
    let rc = unsafe { mprotect(addr, len, prot_bits(plan.prot)) };
    if rc != 0 {
        return Err(LoaderError {});
    }
    Ok(())
}

fn call_constructor(pc: u64) {
    let f: extern "C" fn() = unsafe { std::mem::transmute(pc as usize) };
    f();
}

fn call_destructor(pc: u64) {
    let f: extern "C" fn() = unsafe { std::mem::transmute(pc as usize) };
    f();
}

fn alloc_initial_stack(plan: &LoaderOutput) -> Result<*mut usize, LoaderError> {
    let mut argv0 = if let Some(main_obj) = plan.parsed.first() {
        main_obj.input_name.clone()
    } else {
        b"program".to_vec()
    };
    if argv0.is_empty() || argv0[0] == 0 {
        argv0 = b"program".to_vec();
    }
    if argv0.last().copied() != Some(0) {
        argv0.push(0);
    }

    let stack_words = 7usize;
    let stack_table_bytes = stack_words * std::mem::size_of::<usize>();
    if argv0.len() + stack_table_bytes > STACK_SIZE {
        return Err(LoaderError {});
    }

    let mapped = unsafe {
        mmap(
            ptr::null_mut(),
            STACK_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_STACK,
            -1,
            0,
        )
    };
    if mapped as isize == -1 {
        return Err(LoaderError {});
    }

    let top = (mapped as usize + STACK_SIZE) & !0xfusize;
    let argv0_addr = top - argv0.len();
    let table_top = argv0_addr & !0xfusize;
    let sp = unsafe { (table_top as *mut usize).sub(stack_words) };

    unsafe {
        ptr::copy_nonoverlapping(argv0.as_ptr(), argv0_addr as *mut u8, argv0.len());
        // argc = 1
        ptr::write(sp.add(0), 1);
        // argv[0]
        ptr::write(sp.add(1), argv0_addr);
        // argv[1] = NULL
        ptr::write(sp.add(2), 0);
        // envp[0] = NULL
        ptr::write(sp.add(3), 0);
        // auxv: (AT_NULL, 0)
        ptr::write(sp.add(4), 0);
        ptr::write(sp.add(5), 0);
        // Keep rsp 16-byte alignment expectation used by startup code.
        ptr::write(sp.add(6), 0);
    }

    Ok(sp)
}

pub fn run_runtime(plan: &LoaderOutput) -> Result<(), LoaderError> {
    for m in &plan.mmap_plans {
        map_segment(m)?;
    }

    for m in &plan.mmap_plans {
        protect_segment(m)?;
    }

    let stack_ptr = alloc_initial_stack(plan)?;
    for c in &plan.constructors {
        call_constructor(c.pc);
    }

    unsafe {
        asm!(
            "mov rsp, {stack}",
            "xor rbp, rbp",
            "jmp {entry}",
            stack = in(reg) stack_ptr,
            entry = in(reg) (plan.entry_pc as usize),
            options(noreturn)
        );
    }
}
