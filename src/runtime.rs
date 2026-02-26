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
const RANDOM_LEN: usize = 16;

const ET_EXEC: u16 = 2;
const PT_PHDR: u32 = 6;
const ELF64_PHDR_SIZE: usize = 56;

const AT_NULL: usize = 0;
const AT_PHDR: usize = 3;
const AT_PHENT: usize = 4;
const AT_PHNUM: usize = 5;
const AT_PAGESZ: usize = 6;
const AT_BASE: usize = 7;
const AT_FLAGS: usize = 8;
const AT_ENTRY: usize = 9;
const AT_UID: usize = 11;
const AT_EUID: usize = 12;
const AT_GID: usize = 13;
const AT_EGID: usize = 14;
const AT_HWCAP: usize = 16;
const AT_CLKTCK: usize = 17;
const AT_SECURE: usize = 23;
const AT_RANDOM: usize = 25;
const AT_HWCAP2: usize = 26;
const AT_EXECFN: usize = 31;
const AT_SYSINFO: usize = 32;
const AT_SYSINFO_EHDR: usize = 33;

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
    fn getauxval(t: usize) -> usize;
    fn getuid() -> u32;
    fn geteuid() -> u32;
    fn getgid() -> u32;
    fn getegid() -> u32;
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

fn main_base(plan: &LoaderOutput) -> u64 {
    let Some(main_obj) = plan.parsed.first() else {
        return 0;
    };
    if main_obj.elf_type == ET_EXEC {
        return 0;
    }

    let mut base = u64::MAX;
    for m in &plan.mmap_plans {
        if m.object_name == main_obj.input_name && m.start < base {
            base = m.start;
        }
    }
    if base == u64::MAX {
        0
    } else {
        base
    }
}

fn main_phdr_addr(plan: &LoaderOutput, base: u64) -> u64 {
    let Some(main_obj) = plan.parsed.first() else {
        return 0;
    };
    for ph in &main_obj.phdrs {
        if ph.p_type == PT_PHDR {
            return base.saturating_add(ph.p_vaddr);
        }
    }
    0
}

fn alloc_initial_stack(
    plan: &LoaderOutput,
) -> Result<(*mut usize, *mut *mut i8, *mut i8), LoaderError> {
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
    let random_addr = argv0_addr - RANDOM_LEN;
    let table_top = random_addr & !0xfusize;

    let base = main_base(plan);
    let phdr_addr = main_phdr_addr(plan, base) as usize;
    let phnum = plan.parsed.first().map_or(0, |o| o.phdrs.len());

    let mut random_bytes = [0u8; RANDOM_LEN];
    let host_random = unsafe { getauxval(AT_RANDOM) as *const u8 };
    if !host_random.is_null() {
        unsafe {
            ptr::copy_nonoverlapping(host_random, random_bytes.as_mut_ptr(), RANDOM_LEN);
        }
    }

    let mut auxv: Vec<(usize, usize)> = Vec::new();
    if phdr_addr != 0 && phnum != 0 {
        auxv.push((AT_PHDR, phdr_addr));
        auxv.push((AT_PHENT, ELF64_PHDR_SIZE));
        auxv.push((AT_PHNUM, phnum));
    }
    auxv.push((AT_PAGESZ, 4096));
    auxv.push((AT_BASE, 0));
    auxv.push((AT_FLAGS, 0));
    auxv.push((AT_ENTRY, plan.entry_pc as usize));
    auxv.push((AT_UID, unsafe { getuid() as usize }));
    auxv.push((AT_EUID, unsafe { geteuid() as usize }));
    auxv.push((AT_GID, unsafe { getgid() as usize }));
    auxv.push((AT_EGID, unsafe { getegid() as usize }));
    auxv.push((AT_SECURE, 0));
    auxv.push((AT_RANDOM, random_addr));
    auxv.push((AT_EXECFN, argv0_addr));

    let hwcap = unsafe { getauxval(AT_HWCAP) };
    if hwcap != 0 {
        auxv.push((AT_HWCAP, hwcap));
    }
    let hwcap2 = unsafe { getauxval(AT_HWCAP2) };
    if hwcap2 != 0 {
        auxv.push((AT_HWCAP2, hwcap2));
    }
    let clktck = unsafe { getauxval(AT_CLKTCK) };
    if clktck != 0 {
        auxv.push((AT_CLKTCK, clktck));
    }
    let sysinfo = unsafe { getauxval(AT_SYSINFO) };
    if sysinfo != 0 {
        auxv.push((AT_SYSINFO, sysinfo));
    }
    let sysinfo_ehdr = unsafe { getauxval(AT_SYSINFO_EHDR) };
    if sysinfo_ehdr != 0 {
        auxv.push((AT_SYSINFO_EHDR, sysinfo_ehdr));
    }

    let fixed_words = 4usize;
    let aux_words = auxv.len() * 2 + 2;
    let mut stack_words = fixed_words + aux_words;
    if stack_words % 2 != 0 {
        stack_words += 1;
    }

    let stack_table_bytes = stack_words * std::mem::size_of::<usize>();
    if argv0.len() + RANDOM_LEN + stack_table_bytes > STACK_SIZE {
        return Err(LoaderError {});
    }
    let sp = unsafe { (table_top as *mut usize).sub(stack_words) };

    unsafe {
        ptr::copy_nonoverlapping(
            random_bytes.as_ptr(),
            random_addr as *mut u8,
            RANDOM_LEN,
        );
        ptr::copy_nonoverlapping(argv0.as_ptr(), argv0_addr as *mut u8, argv0.len());

        let mut w = 0usize;
        ptr::write(sp.add(w), 1);
        w += 1;
        ptr::write(sp.add(w), argv0_addr);
        w += 1;
        ptr::write(sp.add(w), 0);
        w += 1;
        ptr::write(sp.add(w), 0);
        w += 1;
        for (k, v) in &auxv {
            ptr::write(sp.add(w), *k);
            w += 1;
            ptr::write(sp.add(w), *v);
            w += 1;
        }
        ptr::write(sp.add(w), AT_NULL);
        w += 1;
        ptr::write(sp.add(w), 0);
        w += 1;
        while w < stack_words {
            ptr::write(sp.add(w), 0);
            w += 1;
        }
    }

    let envp = unsafe { sp.add(3) } as *mut *mut i8;
    let pn = argv0_addr as *mut i8;
    Ok((sp, envp, pn))
}

pub fn run_runtime(plan: &LoaderOutput) -> Result<(), LoaderError> {
    for m in &plan.mmap_plans {
        map_segment(m)?;
    }

    for m in &plan.mmap_plans {
        protect_segment(m)?;
    }

    let (stack_ptr, envp, pn) = alloc_initial_stack(plan)?;
    for c in &plan.constructors {
        let ctor: extern "C" fn(*mut *mut i8, *mut i8) =
            unsafe { std::mem::transmute(c.pc as usize) };
        ctor(envp, pn);
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
