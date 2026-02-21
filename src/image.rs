use std::ptr;

use crate::arith_lemmas::checked_sub_u64;
use crate::model::{LoadPlan, Module};
use crate::rt;
use crate::spec;

pub fn materialize_image(file: &[u8], plan: &LoadPlan) -> *mut u8 {
    let image = rt::rt_mmap_rw(plan.image_len);

    for seg in &plan.segments {
        let dst_off_u64 = checked_sub_u64(seg.vaddr, plan.min_vaddr_page, "segment dst_off");
        let dst_off = rt::checked_usize_from_u64(dst_off_u64, "segment dst_off");

        let file_start = rt::checked_usize_from_u64(seg.fileoff, "segment fileoff");
        let file_size = rt::checked_usize_from_u64(seg.filesz, "segment filesz");
        let file_end = file_start
            .checked_add(file_size)
            .unwrap_or_else(|| rt::fatal("segment file range overflow"));
        if file_end > file.len() {
            rt::fatal(format!(
                "segment file range out of bounds: start={} size={} file_len={}",
                file_start,
                file_size,
                file.len()
            ));
        }

        let dst_file_end = dst_off
            .checked_add(file_size)
            .unwrap_or_else(|| rt::fatal("segment image copy range overflow"));
        if dst_file_end > plan.image_len {
            rt::fatal(format!(
                "segment copy target out of bounds: off={} size={} image_len={}",
                dst_off, file_size, plan.image_len
            ));
        }

        unsafe {
            ptr::copy_nonoverlapping(
                file[file_start..file_end].as_ptr(),
                image.add(dst_off),
                file_size,
            );
        }

        let bss_size_u64 = seg
            .memsz
            .checked_sub(seg.filesz)
            .unwrap_or_else(|| rt::fatal("segment memsz < filesz during zero fill"));
        if bss_size_u64 > 0 {
            let bss_off_u64 = seg
                .vaddr
                .checked_add(seg.filesz)
                .and_then(|v| v.checked_sub(plan.min_vaddr_page))
                .unwrap_or_else(|| rt::fatal("segment BSS offset overflow"));
            let bss_off = rt::checked_usize_from_u64(bss_off_u64, "segment bss off");
            let bss_size = rt::checked_usize_from_u64(bss_size_u64, "segment bss size");

            let bss_end = bss_off
                .checked_add(bss_size)
                .unwrap_or_else(|| rt::fatal("segment BSS range overflow"));
            if bss_end > plan.image_len {
                rt::fatal(format!(
                    "segment BSS target out of bounds: off={} size={} image_len={}",
                    bss_off, bss_size, plan.image_len
                ));
            }

            unsafe {
                ptr::write_bytes(image.add(bss_off), 0u8, bss_size);
            }
        }
    }

    image
}

fn checked_off_common(module: &Module, va: u64, size: u64, require_writable: bool) -> usize {
    if size == 0 {
        rt::fatal("checked_off_common called with size=0");
    }

    if !spec::mapped_range(&module.plan, va, size) {
        rt::fatal(format!(
            "VA range not mapped: module={} va=0x{:x} size=0x{:x}",
            module.path, va, size
        ));
    }

    if require_writable && !spec::writable_range(&module.plan, va, size) {
        rt::fatal(format!(
            "VA range not writable: module={} va=0x{:x} size=0x{:x}",
            module.path, va, size
        ));
    }

    let off = spec::va_to_off(&module.plan, va).unwrap_or_else(|| {
        rt::fatal(format!(
            "va_to_off failed: module={} va=0x{:x}",
            module.path, va
        ))
    });

    let size_usize = rt::checked_usize_from_u64(size, "checked_off size");
    if !spec::off_in_image(module.plan.image_len, off, size_usize) {
        rt::fatal(format!(
            "image offset out-of-bounds: module={} off=0x{:x} size=0x{:x} image_len=0x{:x}",
            module.path, off, size, module.plan.image_len
        ));
    }

    off
}

pub fn checked_off(module: &Module, va: u64, size: u64) -> usize {
    checked_off_common(module, va, size, false)
}

pub fn checked_writable_off(module: &Module, va: u64, size: u64) -> usize {
    checked_off_common(module, va, size, true)
}

pub fn read_u8(module: &Module, va: u64) -> u8 {
    let off = checked_off(module, va, 1);
    unsafe { *module.image.add(off) }
}

pub fn read_u16(module: &Module, va: u64) -> u16 {
    let off = checked_off(module, va, 2);
    let mut bytes = [0u8; 2];
    unsafe {
        ptr::copy_nonoverlapping(module.image.add(off), bytes.as_mut_ptr(), 2);
    }
    u16::from_le_bytes(bytes)
}

pub fn read_u32(module: &Module, va: u64) -> u32 {
    let off = checked_off(module, va, 4);
    let mut bytes = [0u8; 4];
    unsafe {
        ptr::copy_nonoverlapping(module.image.add(off), bytes.as_mut_ptr(), 4);
    }
    u32::from_le_bytes(bytes)
}

pub fn read_u64(module: &Module, va: u64) -> u64 {
    let off = checked_off(module, va, 8);
    let mut bytes = [0u8; 8];
    unsafe {
        ptr::copy_nonoverlapping(module.image.add(off), bytes.as_mut_ptr(), 8);
    }
    u64::from_le_bytes(bytes)
}

pub fn read_i64(module: &Module, va: u64) -> i64 {
    let off = checked_off(module, va, 8);
    let mut bytes = [0u8; 8];
    unsafe {
        ptr::copy_nonoverlapping(module.image.add(off), bytes.as_mut_ptr(), 8);
    }
    i64::from_le_bytes(bytes)
}

pub fn read_c_string(module: &Module, va: u64, max_len: u64) -> String {
    if max_len == 0 {
        rt::fatal(format!(
            "c-string has zero max length: module={} va=0x{:x}",
            module.path, va
        ));
    }

    let mut out = Vec::new();
    let mut i = 0u64;
    while i < max_len {
        let cur = va
            .checked_add(i)
            .unwrap_or_else(|| rt::fatal("c-string VA overflow"));
        let b = read_u8(module, cur);
        if b == 0 {
            return String::from_utf8(out).unwrap_or_else(|_| {
                rt::fatal(format!(
                    "invalid UTF-8 in dynamic string table: module={} va=0x{:x}",
                    module.path, va
                ))
            });
        }
        out.push(b);
        i += 1;
    }

    rt::fatal(format!(
        "unterminated c-string: module={} va=0x{:x} max_len={}",
        module.path, va, max_len
    ));
}

pub fn write_u64_checked(module: &Module, va: u64, value: u64) {
    if va % 8 != 0 {
        rt::fatal(format!(
            "unaligned u64 store: module={} va=0x{:x}",
            module.path, va
        ));
    }

    let off = checked_writable_off(module, va, 8);
    let le = value.to_le();
    unsafe {
        ptr::write_unaligned(module.image.add(off) as *mut u64, le);
    }
}
