use crate::model::{
    ArrayInfo, DynamicInfo, Module, TableInfo, DT_GNU_HASH, DT_HASH, DT_INIT, DT_INIT_ARRAY,
    DT_INIT_ARRAYSZ, DT_JMPREL, DT_NEEDED, DT_NULL, DT_PLTREL, DT_PLTREL_RELA, DT_PLTRELSZ, DT_REL,
    DT_RELA, DT_RELAENT, DT_RELASZ, DT_RELENT, DT_RELSZ, DT_SONAME, DT_STRTAB, DT_STRSZ, DT_SYMENT,
    DT_SYMTAB, DT_TEXTREL, DT_VERDEF, DT_VERNEED, DT_VERSYM, DYN_SIZE, RELA_SIZE, SYM_SIZE,
};
use crate::image;
use crate::rt;
use crate::spec;

pub struct ParsedDynamic {
    pub info: DynamicInfo,
    pub needed: Vec<String>,
    pub soname: Option<String>,
}

pub fn parse_dynamic(module: &Module) -> ParsedDynamic {
    if module.plan.dynamic_memsz == 0 {
        rt::fatal(format!("PT_DYNAMIC has zero size: module={}", module.path));
    }
    if module.plan.dynamic_memsz % DYN_SIZE != 0 {
        rt::fatal(format!(
            "PT_DYNAMIC size is not multiple of {}: module={} size=0x{:x}",
            DYN_SIZE, module.path, module.plan.dynamic_memsz
        ));
    }

    let mut strtab: Option<u64> = None;
    let mut strsz: Option<u64> = None;
    let mut symtab: Option<u64> = None;
    let mut syment: Option<u64> = None;

    let mut rela_addr: Option<u64> = None;
    let mut rela_sz: Option<u64> = None;
    let mut rela_ent: Option<u64> = None;

    let mut jmprel_addr: Option<u64> = None;
    let mut pltrelsz: Option<u64> = None;
    let mut pltrel_kind: Option<u64> = None;

    let mut init: Option<u64> = None;
    let mut init_array_addr: Option<u64> = None;
    let mut init_array_sz: Option<u64> = None;

    let mut soname_off: Option<u64> = None;
    let mut needed_offsets: Vec<u64> = Vec::new();

    let mut dt_hash: Option<u64> = None;

    let dyn_count = module.plan.dynamic_memsz / DYN_SIZE;
    for i in 0..dyn_count {
        let ent_va = module
            .plan
            .dynamic_vaddr
            .checked_add(i * DYN_SIZE)
            .unwrap_or_else(|| rt::fatal("dynamic entry VA overflow"));

        let tag = image::read_i64(module, ent_va);
        let val = image::read_u64(module, ent_va + 8);

        match tag {
            DT_NULL => break,
            DT_NEEDED => needed_offsets.push(val),
            DT_STRTAB => strtab = Some(val),
            DT_STRSZ => strsz = Some(val),
            DT_SYMTAB => symtab = Some(val),
            DT_SYMENT => syment = Some(val),
            DT_RELA => rela_addr = Some(val),
            DT_RELASZ => rela_sz = Some(val),
            DT_RELAENT => rela_ent = Some(val),
            DT_JMPREL => jmprel_addr = Some(val),
            DT_PLTRELSZ => pltrelsz = Some(val),
            DT_PLTREL => pltrel_kind = Some(val),
            DT_INIT => init = Some(val),
            DT_INIT_ARRAY => init_array_addr = Some(val),
            DT_INIT_ARRAYSZ => init_array_sz = Some(val),
            DT_SONAME => soname_off = Some(val),
            DT_HASH => dt_hash = Some(val),
            DT_REL | DT_RELSZ | DT_RELENT => {
                rt::fatal(format!(
                    "DT_REL* is unsupported in MVP: module={} tag={}",
                    module.path, tag
                ));
            }
            DT_TEXTREL => {
                rt::fatal(format!("DT_TEXTREL is unsupported: module={}", module.path));
            }
            DT_VERSYM | DT_VERDEF | DT_VERNEED => {
                rt::fatal(format!(
                    "symbol versioning tags are unsupported: module={} tag=0x{:x}",
                    module.path, tag
                ));
            }
            DT_GNU_HASH => {
                // Optional. We do not require GNU hash lookup in MVP.
            }
            _ => {
                // Other tags are ignored by MVP loader.
            }
        }
    }

    let strtab = strtab.unwrap_or_else(|| rt::fatal(format!("missing DT_STRTAB: module={}", module.path)));
    let strsz = strsz.unwrap_or_else(|| rt::fatal(format!("missing DT_STRSZ: module={}", module.path)));
    let symtab = symtab.unwrap_or_else(|| rt::fatal(format!("missing DT_SYMTAB: module={}", module.path)));
    let syment = syment.unwrap_or_else(|| rt::fatal(format!("missing DT_SYMENT: module={}", module.path)));

    if syment != SYM_SIZE {
        rt::fatal(format!(
            "unexpected DT_SYMENT: module={} syment={} expected={}",
            module.path, syment, SYM_SIZE
        ));
    }

    if strsz == 0 {
        rt::fatal(format!("DT_STRSZ is zero: module={}", module.path));
    }

    if !spec::mapped_range(&module.plan, strtab, strsz) {
        rt::fatal(format!(
            "strtab range not mapped: module={} strtab=0x{:x} strsz=0x{:x}",
            module.path, strtab, strsz
        ));
    }

    let rela = if let Some(addr) = rela_addr {
        let size = rela_sz.unwrap_or_else(|| {
            rt::fatal(format!("DT_RELA present but DT_RELASZ missing: module={}", module.path))
        });
        let ent = rela_ent.unwrap_or_else(|| {
            rt::fatal(format!("DT_RELA present but DT_RELAENT missing: module={}", module.path))
        });
        if ent != RELA_SIZE {
            rt::fatal(format!(
                "unexpected DT_RELAENT: module={} relaent={} expected={}",
                module.path, ent, RELA_SIZE
            ));
        }
        if size % ent != 0 {
            rt::fatal(format!(
                "DT_RELASZ is not multiple of DT_RELAENT: module={} relasz=0x{:x} relaent=0x{:x}",
                module.path, size, ent
            ));
        }
        if size > 0 && !spec::mapped_range(&module.plan, addr, size) {
            rt::fatal(format!(
                "RELA table not mapped: module={} addr=0x{:x} size=0x{:x}",
                module.path, addr, size
            ));
        }
        Some(TableInfo { addr, size, ent })
    } else {
        None
    };

    let jmprel = if let Some(addr) = jmprel_addr {
        let size = pltrelsz.unwrap_or_else(|| {
            rt::fatal(format!(
                "DT_JMPREL present but DT_PLTRELSZ missing: module={}",
                module.path
            ))
        });
        let kind = pltrel_kind.unwrap_or_else(|| {
            rt::fatal(format!(
                "DT_JMPREL present but DT_PLTREL missing: module={}",
                module.path
            ))
        });
        if kind != DT_PLTREL_RELA {
            rt::fatal(format!(
                "DT_PLTREL is not DT_RELA: module={} kind={}",
                module.path, kind
            ));
        }
        if size % RELA_SIZE != 0 {
            rt::fatal(format!(
                "DT_PLTRELSZ is not multiple of RELA entry size: module={} size=0x{:x}",
                module.path, size
            ));
        }
        if size > 0 && !spec::mapped_range(&module.plan, addr, size) {
            rt::fatal(format!(
                "JMPREL table not mapped: module={} addr=0x{:x} size=0x{:x}",
                module.path, addr, size
            ));
        }
        Some(TableInfo {
            addr,
            size,
            ent: RELA_SIZE,
        })
    } else {
        None
    };

    let init_array = match (init_array_addr, init_array_sz) {
        (Some(addr), Some(size)) => {
            if size % 8 != 0 {
                rt::fatal(format!(
                    "DT_INIT_ARRAYSZ is not multiple of 8: module={} size=0x{:x}",
                    module.path, size
                ));
            }
            if size > 0 && !spec::mapped_range(&module.plan, addr, size) {
                rt::fatal(format!(
                    "init_array not mapped: module={} addr=0x{:x} size=0x{:x}",
                    module.path, addr, size
                ));
            }
            Some(ArrayInfo { addr, size })
        }
        (None, None) => None,
        _ => {
            rt::fatal(format!(
                "DT_INIT_ARRAY and DT_INIT_ARRAYSZ must appear together: module={}",
                module.path
            ));
        }
    };

    let sym_count = compute_sym_count(
        module,
        dt_hash,
        symtab,
        syment,
        strtab,
        rela.as_ref(),
        jmprel.as_ref(),
        init_array.as_ref(),
    );

    let sym_bytes = (sym_count as u64)
        .checked_mul(syment)
        .unwrap_or_else(|| rt::fatal("dynsym byte-size overflow"));
    if !spec::mapped_range(&module.plan, symtab, sym_bytes) {
        rt::fatal(format!(
            "dynsym table not mapped: module={} symtab=0x{:x} bytes=0x{:x}",
            module.path, symtab, sym_bytes
        ));
    }

    let mut needed = Vec::with_capacity(needed_offsets.len());
    for &off in &needed_offsets {
        needed.push(read_dyn_string(module, strtab, strsz, off));
    }

    let soname = soname_off.map(|off| read_dyn_string(module, strtab, strsz, off));

    let info = DynamicInfo {
        strtab,
        strsz,
        symtab,
        syment,
        sym_count,
        rela,
        jmprel,
        needed_offsets,
        init,
        init_array,
        soname_off,
    };

    ParsedDynamic { info, needed, soname }
}

fn read_dyn_string(module: &Module, strtab: u64, strsz: u64, off: u64) -> String {
    if off >= strsz {
        rt::fatal(format!(
            "dynamic string offset out of range: module={} off=0x{:x} strsz=0x{:x}",
            module.path, off, strsz
        ));
    }
    let start = strtab
        .checked_add(off)
        .unwrap_or_else(|| rt::fatal("dynamic string start overflow"));
    let max_len = strsz - off;
    image::read_c_string(module, start, max_len)
}

fn compute_sym_count(
    module: &Module,
    dt_hash: Option<u64>,
    symtab: u64,
    syment: u64,
    strtab: u64,
    rela: Option<&TableInfo>,
    jmprel: Option<&TableInfo>,
    init_array: Option<&ArrayInfo>,
) -> usize {
    if let Some(hash_va) = dt_hash {
        if !spec::mapped_range(&module.plan, hash_va, 8) {
            rt::fatal(format!(
                "DT_HASH header not mapped: module={} hash=0x{:x}",
                module.path, hash_va
            ));
        }
        let nchain = image::read_u32(module, hash_va + 4) as usize;
        if nchain == 0 {
            rt::fatal(format!("DT_HASH nchain is zero: module={}", module.path));
        }
        return nchain;
    }

    let mut end = u64::MAX;
    let mut try_update = |candidate: u64| {
        if candidate > symtab && candidate < end {
            end = candidate;
        }
    };

    try_update(strtab);
    if let Some(t) = rela {
        try_update(t.addr);
    }
    if let Some(t) = jmprel {
        try_update(t.addr);
    }
    if let Some(arr) = init_array {
        try_update(arr.addr);
    }

    if end == u64::MAX {
        for seg in &module.plan.segments {
            let seg_end = seg
                .vaddr
                .checked_add(seg.memsz)
                .unwrap_or_else(|| rt::fatal("segment end overflow during sym_count"));
            if symtab >= seg.vaddr && symtab < seg_end {
                end = seg_end;
                break;
            }
        }
    }

    if end == u64::MAX || end <= symtab {
        rt::fatal(format!(
            "could not infer dynsym bound: module={} symtab=0x{:x}",
            module.path, symtab
        ));
    }

    let bytes = end
        .checked_sub(symtab)
        .unwrap_or_else(|| rt::fatal("dynsym bound underflow"));
    if bytes < syment {
        rt::fatal(format!(
            "dynsym bound too small: module={} bytes=0x{:x} syment=0x{:x}",
            module.path, bytes, syment
        ));
    }

    let count = (bytes / syment) as usize;
    if count == 0 {
        rt::fatal(format!("computed dynsym count is zero: module={}", module.path));
    }
    count
}
