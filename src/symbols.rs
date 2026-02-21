use crate::arith_lemmas::checked_add_u64;
use crate::image;
use crate::model::{
    Module, Symbol, SHN_UNDEF, STB_GLOBAL, STB_LOCAL, STB_WEAK,
};
use crate::rt;

fn symbol_binding(info: u8) -> u8 {
    info >> 4
}

pub fn read_symbol(module: &Module, sym_index: u32) -> Symbol {
    let idx = sym_index as usize;
    if idx >= module.dynamic.sym_count {
        rt::fatal(format!(
            "symbol index out of bounds: module={} index={} count={}",
            module.path, idx, module.dynamic.sym_count
        ));
    }

    let off = (idx as u64)
        .checked_mul(module.dynamic.syment)
        .unwrap_or_else(|| rt::fatal("symbol offset overflow"));
    let va = checked_add_u64(module.dynamic.symtab, off, "symbol VA");

    Symbol {
        name_off: image::read_u32(module, va),
        info: image::read_u8(module, va + 4),
        other: image::read_u8(module, va + 5),
        shndx: image::read_u16(module, va + 6),
        value: image::read_u64(module, va + 8),
        size: image::read_u64(module, va + 16),
    }
}

pub fn symbol_name(module: &Module, sym: &Symbol) -> String {
    let name_off = sym.name_off as u64;
    if name_off >= module.dynamic.strsz {
        rt::fatal(format!(
            "symbol name offset out of range: module={} off=0x{:x} strsz=0x{:x}",
            module.path, name_off, module.dynamic.strsz
        ));
    }

    let va = module
        .dynamic
        .strtab
        .checked_add(name_off)
        .unwrap_or_else(|| rt::fatal("symbol name VA overflow"));
    image::read_c_string(module, va, module.dynamic.strsz - name_off)
}

pub fn resolve_symbol(
    modules: &[Module],
    scope_order: &[usize],
    requester_idx: usize,
    sym_index: u32,
) -> u64 {
    let requester = &modules[requester_idx];
    let sym = read_symbol(requester, sym_index);
    let bind = symbol_binding(sym.info);

    if bind == STB_LOCAL {
        if sym.shndx == SHN_UNDEF {
            rt::fatal(format!(
                "local symbol is undefined: module={} index={}",
                requester.path, sym_index
            ));
        }
        return requester
            .base
            .checked_add(sym.value)
            .unwrap_or_else(|| rt::fatal("local symbol address overflow"));
    }

    let name = symbol_name(requester, &sym);
    for &scope_idx in scope_order {
        let module = &modules[scope_idx];

        for i in 0..module.dynamic.sym_count {
            let cand = read_symbol(module, i as u32);
            if cand.shndx == SHN_UNDEF {
                continue;
            }

            let cand_bind = symbol_binding(cand.info);
            if cand_bind == STB_WEAK {
                continue;
            }
            if cand_bind != STB_GLOBAL {
                continue;
            }

            let cand_name = symbol_name(module, &cand);
            if cand_name == name {
                return module
                    .base
                    .checked_add(cand.value)
                    .unwrap_or_else(|| rt::fatal("global symbol address overflow"));
            }
        }
    }

    rt::fatal(format!(
        "unresolved symbol: requester={} index={} name={}",
        requester.path, sym_index, name
    ));
}
