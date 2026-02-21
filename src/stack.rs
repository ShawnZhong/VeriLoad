use std::ptr;

use crate::rt;

const STACK_SIZE: usize = 128 * 1024;

pub fn build_minimal_stack(program_path: &str) -> *const u8 {
    let stack = rt::rt_mmap_rw(STACK_SIZE);
    let base = stack as usize;
    let top = base
        .checked_add(STACK_SIZE)
        .unwrap_or_else(|| rt::fatal("stack top overflow"));

    let path_bytes = program_path.as_bytes();
    let mut cursor = top;

    let path_total = path_bytes
        .len()
        .checked_add(1)
        .unwrap_or_else(|| rt::fatal("program path length overflow"));
    if path_total + 64 > STACK_SIZE {
        rt::fatal(format!(
            "program path too long for startup stack: len={} max={} path={}",
            path_bytes.len(),
            STACK_SIZE - 64,
            program_path
        ));
    }

    cursor = cursor
        .checked_sub(path_total)
        .unwrap_or_else(|| rt::fatal("stack underflow while writing argv[0]"));

    unsafe {
        ptr::copy_nonoverlapping(path_bytes.as_ptr(), cursor as *mut u8, path_bytes.len());
        *((cursor as *mut u8).add(path_bytes.len())) = 0;
    }

    let argv0_ptr = cursor as u64;

    cursor &= !0x0f;

    push_u64(&mut cursor, base, 0); // envp = NULL
    push_u64(&mut cursor, base, 0); // argv[1] = NULL
    push_u64(&mut cursor, base, argv0_ptr); // argv[0]
    push_u64(&mut cursor, base, 1); // argc

    cursor as *const u8
}

fn push_u64(cursor: &mut usize, stack_base: usize, value: u64) {
    *cursor = cursor
        .checked_sub(8)
        .unwrap_or_else(|| rt::fatal("startup stack underflow"));

    if *cursor < stack_base {
        rt::fatal("startup stack write before base");
    }

    unsafe {
        ptr::write_unaligned(*cursor as *mut u64, value);
    }
}
