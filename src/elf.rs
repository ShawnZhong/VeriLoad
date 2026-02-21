use std::path::Path;

use crate::rt_common::*;

fn find_load_segment_containing_vaddr(parsed: &ParsedObject, vaddr: u64) -> Option<&ProgramHeader> {
    for ph in &parsed.program_headers {
        if ph.p_type != PT_LOAD {
            continue;
        }
        if vaddr >= ph.p_vaddr && vaddr < ph.p_vaddr.saturating_add(ph.p_filesz) {
            return Some(ph);
        }
    }
    None
}

fn vaddr_to_file_off(parsed: &ParsedObject, vaddr: u64) -> usize {
    match find_load_segment_containing_vaddr(parsed, vaddr) {
        Some(ph) => {
            let delta = vaddr - ph.p_vaddr;
            let off = ph.p_offset.saturating_add(delta);
            if off > usize::MAX as u64 {
                fatal("file offset overflow");
            }
            off as usize
        },
        None => fatal("virtual address not covered by PT_LOAD file range"),
    }
}

fn parse_program_headers(bytes: &[u8], phoff: u64, phnum: u16, phentsize: u16) -> Vec<ProgramHeader> {
    if (phentsize as usize) < 56 {
        fatal("ELF64 program header entry too small");
    }
    let mut out: Vec<ProgramHeader> = Vec::new();
    for i in 0..phnum {
        let off64 = phoff.saturating_add((i as u64).saturating_mul(phentsize as u64));
        if off64 > usize::MAX as u64 {
            fatal("program header offset overflow");
        }
        let off = off64 as usize;
        if off + (phentsize as usize) > bytes.len() {
            fatal("truncated ELF program header table");
        }
        let ph = ProgramHeader {
            p_type: read_u32(bytes, off),
            p_flags: read_u32(bytes, off + 4),
            p_offset: read_u64(bytes, off + 8),
            p_vaddr: read_u64(bytes, off + 16),
            p_filesz: read_u64(bytes, off + 32),
            p_memsz: read_u64(bytes, off + 40),
            p_align: read_u64(bytes, off + 48),
        };
        out.push(ph);
    }
    out
}

fn parse_dynamic_entries(parsed: &ParsedObject) -> Vec<DynamicEntry> {
    let dyn_ph = parsed.program_headers.iter().find(|ph| ph.p_type == PT_DYNAMIC);
    let mut entries: Vec<DynamicEntry> = Vec::new();
    let Some(ph) = dyn_ph else {
        return entries;
    };
    let start64 = ph.p_offset;
    let size64 = ph.p_filesz;
    if start64 > usize::MAX as u64 || size64 > usize::MAX as u64 {
        fatal("dynamic section bounds overflow");
    }
    let start = start64 as usize;
    let size = size64 as usize;
    if start + size > parsed.bytes.len() {
        fatal("PT_DYNAMIC outside file bounds");
    }
    let mut off = start;
    let end = start + size;
    while off + 16 <= end {
        let tag = read_i64(&parsed.bytes, off);
        let value = read_u64(&parsed.bytes, off + 8);
        entries.push(DynamicEntry { tag, value });
        off += 16;
        if tag == DT_NULL {
            break;
        }
    }
    entries
}

pub fn parse_elf(id: usize, path: &Path, bytes: Vec<u8>) -> ParsedObject {
    if bytes.len() < 64 {
        fatal("ELF header too small");
    }
    if bytes[0] != 0x7f || bytes[1] != b'E' || bytes[2] != b'L' || bytes[3] != b'F' {
        fatal("input is not ELF");
    }
    if bytes[4] != 2 {
        fatal("only ELF64 is supported");
    }
    if bytes[5] != 1 {
        fatal("only little-endian ELF is supported");
    }
    if bytes[6] != 1 {
        fatal("unsupported ELF version");
    }

    let elf_type = read_u16(&bytes, 16);
    let machine = read_u16(&bytes, 18);
    let entry = read_u64(&bytes, 24);
    let phoff = read_u64(&bytes, 32);
    let phentsize = read_u16(&bytes, 54);
    let phnum = read_u16(&bytes, 56);

    if machine != 62 {
        fatal("only x86_64 ELF is supported");
    }
    if phnum == 0 {
        fatal("ELF has no program headers");
    }

    let program_headers = parse_program_headers(&bytes, phoff, phnum, phentsize);
    if !program_headers.iter().any(|ph| ph.p_type == PT_LOAD) {
        fatal("ELF has no PT_LOAD segment");
    }

    let mut prev_vaddr: Option<u64> = None;
    for ph in &program_headers {
        if ph.p_type != PT_LOAD {
            continue;
        }
        if ph.p_filesz > ph.p_memsz {
            fatal("PT_LOAD p_filesz > p_memsz");
        }
        if ph.p_offset.saturating_add(ph.p_filesz) > bytes.len() as u64 {
            fatal("PT_LOAD file range outside file");
        }
        if ph.p_vaddr % PAGE_SIZE != ph.p_offset % PAGE_SIZE {
            fatal("PT_LOAD p_vaddr and p_offset are not congruent modulo page size");
        }
        if let Some(prev) = prev_vaddr {
            if prev > ph.p_vaddr {
                fatal("PT_LOAD segments are not sorted by p_vaddr");
            }
        }
        prev_vaddr = Some(ph.p_vaddr);
    }

    let interp = match program_headers.iter().find(|ph| ph.p_type == PT_INTERP) {
        Some(ph) => {
            let start = ph.p_offset as usize;
            let size = ph.p_filesz as usize;
            if start + size > bytes.len() {
                fatal("PT_INTERP outside file");
            }
            Some(read_cstr(&bytes[start..start + size]))
        },
        None => None,
    };

    let mut parsed = ParsedObject {
        id,
        name: path_display(path),
        path: path.to_path_buf(),
        bytes,
        entry,
        elf_type,
        machine,
        interp,
        program_headers,
        dynamic: Vec::new(),
        needed: Vec::new(),
        soname: None,
        rpath: None,
        init: None,
        has_bind_now: false,
    };

    parsed.dynamic = parse_dynamic_entries(&parsed);
    if parsed.dynamic.is_empty() {
        return parsed;
    }

    let mut strtab_addr: Option<u64> = None;
    let mut strtab_size: Option<u64> = None;
    for dynent in &parsed.dynamic {
        match dynent.tag {
            DT_STRTAB => strtab_addr = Some(dynent.value),
            DT_STRSZ => strtab_size = Some(dynent.value),
            DT_INIT => parsed.init = Some(dynent.value),
            DT_BIND_NOW => parsed.has_bind_now = true,
            DT_FLAGS => {
                if (dynent.value & DF_BIND_NOW) != 0 {
                    parsed.has_bind_now = true;
                }
            },
            DT_FLAGS_1 => {
                if (dynent.value & DF_1_NOW) != 0 {
                    parsed.has_bind_now = true;
                }
            },
            DT_GNU_HASH => {},
            _ => {},
        }
    }

    let Some(st_addr) = strtab_addr else {
        fatal("missing DT_STRTAB in dynamic object");
    };
    let Some(st_size) = strtab_size else {
        fatal("missing DT_STRSZ in dynamic object");
    };
    let st_off = vaddr_to_file_off(&parsed, st_addr);
    let st_end = st_off.saturating_add(st_size as usize);
    if st_end > parsed.bytes.len() {
        fatal("dynamic string table outside file bounds");
    }
    let strtab = &parsed.bytes[st_off..st_end];

    for dynent in &parsed.dynamic {
        match dynent.tag {
            DT_NEEDED => parsed.needed.push(parse_dyn_string(strtab, dynent.value)),
            DT_SONAME => parsed.soname = Some(parse_dyn_string(strtab, dynent.value)),
            DT_RPATH => parsed.rpath = Some(parse_dyn_string(strtab, dynent.value)),
            _ => {},
        }
    }

    parsed
}
