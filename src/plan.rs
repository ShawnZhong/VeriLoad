use crate::arith_lemmas::{align_down, align_up_checked, checked_add_u64, checked_sub_u64, PAGE_SIZE};
use crate::model::{
    ElfHeader, LoadPlan, ProgramHeader, RelroRegion, Segment, PF_R, PF_W, PF_X, PT_DYNAMIC, PT_GNU_RELRO,
    PT_INTERP, PT_LOAD, PT_NOTE, PT_PHDR, PT_TLS,
};
use crate::rt;
use crate::spec;

pub fn build_load_plan(file_len: usize, ehdr: &ElfHeader, phdrs: &[ProgramHeader]) -> LoadPlan {
    let mut segments: Vec<Segment> = Vec::new();
    let mut dynamic: Option<(u64, u64)> = None;
    let mut relro: Option<RelroRegion> = None;

    for ph in phdrs {
        match ph.p_type {
            PT_LOAD => {
                if ph.p_filesz > ph.p_memsz {
                    rt::fatal(format!(
                        "PT_LOAD filesz > memsz: filesz=0x{:x} memsz=0x{:x}",
                        ph.p_filesz, ph.p_memsz
                    ));
                }

                let file_end = checked_add_u64(ph.p_offset, ph.p_filesz, "PT_LOAD file end");
                if file_end > file_len as u64 {
                    rt::fatal(format!(
                        "PT_LOAD exceeds file bounds: offset=0x{:x} filesz=0x{:x} file_len=0x{:x}",
                        ph.p_offset, ph.p_filesz, file_len
                    ));
                }

                let _mem_end = checked_add_u64(ph.p_vaddr, ph.p_memsz, "PT_LOAD mem end");

                if ph.p_vaddr % PAGE_SIZE != ph.p_offset % PAGE_SIZE {
                    rt::fatal(format!(
                        "PT_LOAD alignment mismatch: vaddr=0x{:x} off=0x{:x}",
                        ph.p_vaddr, ph.p_offset
                    ));
                }

                let pf_known = PF_R | PF_W | PF_X;
                if ph.p_flags & !pf_known != 0 {
                    rt::fatal(format!("PT_LOAD has unknown p_flags bits: 0x{:x}", ph.p_flags));
                }

                segments.push(Segment {
                    vaddr: ph.p_vaddr,
                    memsz: ph.p_memsz,
                    filesz: ph.p_filesz,
                    fileoff: ph.p_offset,
                    flags: ph.p_flags,
                });
            }
            PT_DYNAMIC => {
                if dynamic.is_some() {
                    rt::fatal("multiple PT_DYNAMIC headers are not supported");
                }
                dynamic = Some((ph.p_vaddr, ph.p_memsz));
            }
            PT_GNU_RELRO => {
                relro = Some(RelroRegion {
                    vaddr: ph.p_vaddr,
                    memsz: ph.p_memsz,
                });
            }
            PT_INTERP | PT_PHDR | PT_NOTE => {
                // Allowed and ignored for loading decisions.
            }
            PT_TLS => {
                rt::fatal("PT_TLS is unsupported in MVP loader");
            }
            _ => {
                // Unknown program headers are ignored.
            }
        }
    }

    if segments.is_empty() {
        rt::fatal("ELF has no PT_LOAD segments");
    }

    let (dynamic_vaddr, dynamic_memsz) = dynamic.unwrap_or_else(|| {
        rt::fatal("ELF has no PT_DYNAMIC segment");
    });

    segments.sort_by_key(|s| s.vaddr);

    for i in 1..segments.len() {
        let prev = &segments[i - 1];
        let curr = &segments[i];
        let prev_end = checked_add_u64(prev.vaddr, prev.memsz, "segment end");
        if prev_end > curr.vaddr {
            rt::fatal(format!(
                "overlapping PT_LOAD segments: prev=[0x{:x},0x{:x}) curr_start=0x{:x}",
                prev.vaddr, prev_end, curr.vaddr
            ));
        }
    }

    let first = &segments[0];
    let min_vaddr_page = align_down(first.vaddr, PAGE_SIZE);

    let mut max_end = 0u64;
    for seg in &segments {
        let end = checked_add_u64(seg.vaddr, seg.memsz, "segment max end");
        if end > max_end {
            max_end = end;
        }
    }
    let max_vaddr_page = align_up_checked(max_end, PAGE_SIZE);
    let span = checked_sub_u64(max_vaddr_page, min_vaddr_page, "image span");
    let image_len = rt::checked_usize_from_u64(span, "image length");

    if image_len == 0 {
        rt::fatal("image length computed as zero");
    }

    let plan = LoadPlan {
        min_vaddr_page,
        max_vaddr_page,
        image_len,
        segments,
        dynamic_vaddr,
        dynamic_memsz,
        relro,
        entry: ehdr.e_entry,
    };

    if dynamic_memsz == 0 {
        rt::fatal("PT_DYNAMIC memsz is zero");
    }
    if !spec::mapped_range(&plan, dynamic_vaddr, dynamic_memsz) {
        rt::fatal(format!(
            "PT_DYNAMIC range not mapped by PT_LOAD segments: va=0x{:x} size=0x{:x}",
            dynamic_vaddr, dynamic_memsz
        ));
    }

    plan
}
