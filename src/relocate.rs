use crate::arith_lemmas::{add_signed_u64, checked_add_u64};
use crate::image;
use crate::model::{
    Module, RelaEntry, TableInfo, R_X86_64_64, R_X86_64_GLOB_DAT, R_X86_64_IRELATIVE,
    R_X86_64_JUMP_SLOT, R_X86_64_RELATIVE,
};
use crate::rt;
use crate::symbols;

pub fn relocate_all(modules: &[Module], dependency_first_order: &[usize]) {
    let global_scope: Vec<usize> = (0..modules.len()).collect();

    for &idx in dependency_first_order {
        relocate_module(modules, &global_scope, idx);
    }
}

fn relocate_module(modules: &[Module], scope_order: &[usize], module_idx: usize) {
    let module = &modules[module_idx];
    let relocs = collect_relocations(module);

    for rela in &relocs {
        let kind = reloc_type(rela.info);
        if kind == R_X86_64_RELATIVE {
            apply_one_relocation(modules, scope_order, module_idx, rela);
        }
    }

    for rela in &relocs {
        let kind = reloc_type(rela.info);
        if kind != R_X86_64_RELATIVE {
            apply_one_relocation(modules, scope_order, module_idx, rela);
        }
    }
}

fn collect_relocations(module: &Module) -> Vec<RelaEntry> {
    let mut out = Vec::new();

    if let Some(table) = &module.dynamic.rela {
        out.extend(read_rela_table(module, table));
    }
    if let Some(table) = &module.dynamic.jmprel {
        out.extend(read_rela_table(module, table));
    }

    out
}

fn read_rela_table(module: &Module, table: &TableInfo) -> Vec<RelaEntry> {
    if table.ent == 0 {
        rt::fatal(format!(
            "RELA table entry size is zero: module={} addr=0x{:x}",
            module.path, table.addr
        ));
    }
    if table.size % table.ent != 0 {
        rt::fatal(format!(
            "RELA table has non-integral count: module={} size=0x{:x} ent=0x{:x}",
            module.path, table.size, table.ent
        ));
    }
    if table.ent != 24 {
        rt::fatal(format!(
            "unexpected RELA entry size: module={} ent=0x{:x}",
            module.path, table.ent
        ));
    }

    let count = (table.size / table.ent) as usize;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let off = (i as u64)
            .checked_mul(table.ent)
            .unwrap_or_else(|| rt::fatal("RELA entry offset overflow"));
        let va = table
            .addr
            .checked_add(off)
            .unwrap_or_else(|| rt::fatal("RELA entry VA overflow"));

        out.push(RelaEntry {
            offset: image::read_u64(module, va),
            info: image::read_u64(module, va + 8),
            addend: image::read_i64(module, va + 16),
        });
    }

    out
}

fn reloc_type(info: u64) -> u32 {
    (info & 0xffff_ffff) as u32
}

fn reloc_sym_index(info: u64) -> u32 {
    (info >> 32) as u32
}

fn apply_one_relocation(
    modules: &[Module],
    scope_order: &[usize],
    module_idx: usize,
    rela: &RelaEntry,
) {
    let module = &modules[module_idx];

    let kind = reloc_type(rela.info);
    if kind == R_X86_64_IRELATIVE {
        rt::fatal(format!(
            "R_X86_64_IRELATIVE is unsupported: module={} offset=0x{:x}",
            module.path, rela.offset
        ));
    }

    let _place = checked_add_u64(module.base, rela.offset, "relocation place");

    let value = match kind {
        R_X86_64_RELATIVE => add_signed_u64(module.base, rela.addend, "R_RELATIVE value"),
        R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
            let sym_index = reloc_sym_index(rela.info);
            symbols::resolve_symbol(modules, scope_order, module_idx, sym_index)
        }
        R_X86_64_64 => {
            let sym_index = reloc_sym_index(rela.info);
            let s = symbols::resolve_symbol(modules, scope_order, module_idx, sym_index);
            add_signed_u64(s, rela.addend, "R_X86_64_64 value")
        }
        _ => {
            rt::fatal(format!(
                "unsupported relocation kind: module={} kind={} offset=0x{:x}",
                module.path, kind, rela.offset
            ));
        }
    };

    image::write_u64_checked(module, rela.offset, value);
}
