use crate::arith_lemmas::{align_down, align_up_checked, checked_add_u64, checked_sub_u64, PAGE_SIZE};
use crate::model::{Module, PF_R, PF_W, PF_X};
use crate::rt;

pub fn apply_all_protections(modules: &[Module]) {
    for module in modules {
        apply_segment_protections(module);
        apply_relro(module);
    }
}

fn prot_from_pflags(flags: u32) -> i32 {
    let mut prot = 0;
    if (flags & PF_R) != 0 {
        prot |= rt::PROT_READ;
    }
    if (flags & PF_W) != 0 {
        prot |= rt::PROT_WRITE;
    }
    if (flags & PF_X) != 0 {
        prot |= rt::PROT_EXEC;
    }
    prot
}

fn apply_segment_protections(module: &Module) {
    for seg in &module.plan.segments {
        if seg.memsz == 0 {
            continue;
        }

        let seg_end = checked_add_u64(seg.vaddr, seg.memsz, "segment end for mprotect");
        let start = align_down(seg.vaddr, PAGE_SIZE);
        let end = align_up_checked(seg_end, PAGE_SIZE);
        let len_u64 = checked_sub_u64(end, start, "segment mprotect len");
        let len = rt::checked_usize_from_u64(len_u64, "segment mprotect len");

        let off_u64 = checked_sub_u64(start, module.plan.min_vaddr_page, "segment mprotect off");
        let off = rt::checked_usize_from_u64(off_u64, "segment mprotect off");

        let end_off = off
            .checked_add(len)
            .unwrap_or_else(|| rt::fatal("segment mprotect range overflow"));
        if end_off > module.plan.image_len {
            rt::fatal(format!(
                "segment mprotect range outside image: module={} off=0x{:x} len=0x{:x} image_len=0x{:x}",
                module.path, off, len, module.plan.image_len
            ));
        }

        let addr = unsafe { module.image.add(off) };
        let prot = prot_from_pflags(seg.flags);
        rt::rt_mprotect(
            addr,
            len,
            prot,
            &format!("segment-prot module={} vaddr=0x{:x}", module.path, seg.vaddr),
        );
    }
}

fn apply_relro(module: &Module) {
    let Some(relro) = &module.plan.relro else {
        return;
    };
    if relro.memsz == 0 {
        return;
    }

    let relro_end = checked_add_u64(relro.vaddr, relro.memsz, "RELRO end");
    let start = align_down(relro.vaddr, PAGE_SIZE);
    let end = align_up_checked(relro_end, PAGE_SIZE);
    let len_u64 = checked_sub_u64(end, start, "RELRO len");
    let len = rt::checked_usize_from_u64(len_u64, "RELRO len");

    let off_u64 = checked_sub_u64(start, module.plan.min_vaddr_page, "RELRO off");
    let off = rt::checked_usize_from_u64(off_u64, "RELRO off");

    let end_off = off
        .checked_add(len)
        .unwrap_or_else(|| rt::fatal("RELRO range overflow"));
    if end_off > module.plan.image_len {
        rt::fatal(format!(
            "RELRO range outside image: module={} off=0x{:x} len=0x{:x} image_len=0x{:x}",
            module.path, off, len, module.plan.image_len
        ));
    }

    let addr = unsafe { module.image.add(off) };
    rt::rt_mprotect(
        addr,
        len,
        rt::PROT_READ,
        &format!("relro module={} vaddr=0x{:x}", module.path, relro.vaddr),
    );
}
