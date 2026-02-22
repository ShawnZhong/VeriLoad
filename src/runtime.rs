use crate::types::{LoaderError, LoaderOutput, MmapPlan, ProtFlags, RelocWrite};
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

fn apply_reloc_write(write: &RelocWrite) {
    let slot = write.write_addr as usize as *mut u64;
    unsafe {
        ptr::write_unaligned(slot, write.value);
    }
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

fn alloc_initial_stack() -> Result<*mut usize, LoaderError> {
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
    let sp = unsafe { (top as *mut usize).sub(5) };

    unsafe {
        // argc
        ptr::write(sp.add(0), 0);
        // argv[0] = NULL
        ptr::write(sp.add(1), 0);
        // envp[0] = NULL
        ptr::write(sp.add(2), 0);
        // auxv terminator: (AT_NULL, 0)
        ptr::write(sp.add(3), 0);
        ptr::write(sp.add(4), 0);
    }

    Ok(sp)
}

#[cfg(target_arch = "x86_64")]
unsafe fn jump_to_entry(entry_pc: u64, stack_ptr: *mut usize) -> ! {
    asm!(
        "mov rsp, {stack}",
        "xor rbp, rbp",
        "jmp {entry}",
        stack = in(reg) stack_ptr,
        entry = in(reg) (entry_pc as usize),
        options(noreturn)
    );
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn jump_to_entry(_entry_pc: u64, _stack_ptr: *mut usize) -> ! {
    std::process::abort()
}

pub fn run_runtime(plan: &LoaderOutput) -> Result<(), LoaderError> {
    for m in &plan.mmap_plans {
        map_segment(m)?;
    }
    for w in &plan.reloc_writes {
        apply_reloc_write(w);
    }
    for m in &plan.mmap_plans {
        protect_segment(m)?;
    }
    for c in &plan.constructors {
        call_constructor(c.pc);
    }

    let sp = alloc_initial_stack()?;
    unsafe { jump_to_entry(plan.entry_pc, sp) }
}
