use crate::consts::*;
use crate::relocate_apply_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn add_u64_or_zero(a: u64, b: u64) -> u64 {
    if a <= u64::MAX - b {
        ((a as int) + (b as int)) as u64
    } else {
        0
    }
}

pub open spec fn add_i64_or_zero(base: u64, addend: i64) -> u64 {
    let sum = (base as i128) + (addend as i128);
    if 0 <= sum && sum <= u64::MAX as i128 {
        sum as u64
    } else {
        0
    }
}

pub open spec fn rela_type_of(r: RelaEntry) -> u32 {
    (r.info & 0xffff_ffff) as u32
}

pub open spec fn dyn_base_for_pos(pos: int) -> u64 {
    if pos < 0 {
        0
    } else {
        let raw = (DYN_BASE_START as i128) + (pos as i128) * (DYN_BASE_STRIDE as i128);
        if 0 <= raw && raw <= u64::MAX as i128 {
            raw as u64
        } else {
            0
        }
    }
}

pub open spec fn pos_to_i128_or_zero(pos: int) -> i128 {
    if pos < 0 || pos > usize::MAX as int {
        0
    } else {
        pos as i128
    }
}

pub open spec fn base_for_load_pos(parsed: Seq<ParsedObject>, order: Seq<usize>, pos: int) -> u64
    recommends
        0 <= pos < order.len(),
        (order[pos] as int) < parsed.len(),
{
    let obj_idx = order[pos] as int;
    if parsed[obj_idx].elf_type == ET_EXEC {
        0
    } else {
        dyn_base_for_pos(pos)
    }
}

pub open spec fn object_base_from(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
    scan: nat,
) -> u64
    decreases order.len() - scan,
{
    if scan >= order.len() {
        0
    } else {
        let cur = order[scan as int] as int;
        if cur == obj_idx && 0 <= cur < parsed.len() {
            base_for_load_pos(parsed, order, scan as int)
        } else {
            object_base_from(parsed, order, obj_idx, (scan + 1) as nat)
        }
    }
}

pub open spec fn object_base(parsed: Seq<ParsedObject>, order: Seq<usize>, obj_idx: int) -> u64 {
    object_base_from(parsed, order, obj_idx, 0)
}

pub open spec fn relative_write_for_entry(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
    rela: RelaEntry,
) -> RelocWrite
    recommends
        0 <= obj_idx < parsed.len(),
        rela_type_of(rela) == R_X86_64_RELATIVE,
{
    RelocWrite {
        object_name: parsed[obj_idx].input_name,
        write_addr: add_u64_or_zero(object_base(parsed, order, obj_idx), rela.offset),
        value: add_i64_or_zero(object_base(parsed, order, obj_idx), rela.addend),
        reloc_type: R_X86_64_RELATIVE,
    }
}

pub open spec fn symbol_write_for_rr(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    rr: ResolvedReloc,
    rela: RelaEntry,
) -> RelocWrite
    recommends
        0 <= (rr.requester as int) && (rr.requester as int) < parsed.len(),
        rela_type_of(rela) == R_X86_64_JUMP_SLOT || rela_type_of(rela) == R_X86_64_GLOB_DAT
            || rela_type_of(rela) == R_X86_64_COPY || rela_type_of(rela) == R_X86_64_64,
{
    let req = rr.requester as int;
    RelocWrite {
        object_name: parsed[req].input_name,
        write_addr: add_u64_or_zero(object_base(parsed, order, req), rela.offset),
        value: if rela_type_of(rela) == R_X86_64_64 {
            add_i64_or_zero(rr_provider_value(parsed, order, rr), rela.addend)
        } else {
            rr_provider_value(parsed, order, rr)
        },
        reloc_type: rela_type_of(rela),
    }
}

pub open spec fn expected_relative_rela_writes_from(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
    i: nat,
) -> Seq<RelocWrite>
    decreases if 0 <= obj_idx < parsed.len() {
        parsed[obj_idx].relas@.len() - i
    } else {
        0
    },
{
    if !(0 <= obj_idx < parsed.len()) {
        Seq::empty()
    } else if i >= parsed[obj_idx].relas@.len() {
        Seq::empty()
    } else {
        let rela = parsed[obj_idx].relas@[i as int];
        let tail = expected_relative_rela_writes_from(parsed, order, obj_idx, (i + 1) as nat);
        if rela_type_of(rela) == R_X86_64_RELATIVE {
            seq![relative_write_for_entry(parsed, order, obj_idx, rela)] + tail
        } else {
            tail
        }
    }
}

pub open spec fn expected_relative_jmprel_writes_from(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
    i: nat,
) -> Seq<RelocWrite>
    decreases if 0 <= obj_idx < parsed.len() {
        parsed[obj_idx].jmprels@.len() - i
    } else {
        0
    },
{
    if !(0 <= obj_idx < parsed.len()) {
        Seq::empty()
    } else if i >= parsed[obj_idx].jmprels@.len() {
        Seq::empty()
    } else {
        let rela = parsed[obj_idx].jmprels@[i as int];
        let tail = expected_relative_jmprel_writes_from(parsed, order, obj_idx, (i + 1) as nat);
        if rela_type_of(rela) == R_X86_64_RELATIVE {
            seq![relative_write_for_entry(parsed, order, obj_idx, rela)] + tail
        } else {
            tail
        }
    }
}

pub open spec fn expected_relative_writes_for_obj(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
) -> Seq<RelocWrite> {
    expected_relative_rela_writes_from(parsed, order, obj_idx, 0)
        + expected_relative_jmprel_writes_from(parsed, order, obj_idx, 0)
}

pub open spec fn expected_relative_writes_from_order(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    pos: nat,
) -> Seq<RelocWrite>
    decreases order.len() - pos,
{
    if pos >= order.len() {
        Seq::empty()
    } else {
        let obj_idx = order[pos as int] as int;
        expected_relative_writes_for_obj(parsed, order, obj_idx)
            + expected_relative_writes_from_order(parsed, order, (pos + 1) as nat)
    }
}

pub open spec fn expected_symbol_writes_from(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    resolved: ResolutionResult,
    i: nat,
) -> Seq<RelocWrite>
    decreases resolved.resolved_relocs@.len() - i,
{
    if i >= resolved.resolved_relocs@.len() {
        Seq::empty()
    } else {
        let rr = resolved.resolved_relocs@[i as int];
        let tail = expected_symbol_writes_from(parsed, order, resolved, (i + 1) as nat);
        match rr_reloc_entry(parsed, rr) {
            Some(rel) => {
                let rel_type = rela_type_of(rel);
                let req = rr.requester as int;
                if (rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT
                    || rel_type == R_X86_64_64)
                    && 0 <= req < parsed.len()
                {
                    seq![symbol_write_for_rr(parsed, order, rr, rel)] + tail
                } else if rel_type == R_X86_64_COPY
                    && 0 <= req < parsed.len()
                {
                    seq![symbol_write_for_rr(parsed, order, rr, rel)] + tail
                } else {
                    tail
                }
            }
            None => tail,
        }
    }
}

pub open spec fn expected_reloc_writes(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    resolved: ResolutionResult,
) -> Seq<RelocWrite> {
    expected_relative_writes_from_order(parsed, order, 0)
        + expected_symbol_writes_from(parsed, order, resolved, 0)
}

pub open spec fn plan_relocate_stage_spec(
    parsed: Seq<ParsedObject>,
    discovered: DiscoveryResult,
    resolved: ResolutionResult,
    mmap_plans: Seq<MmapPlan>,
    out: RelocatePlanOutput,
) -> bool {
    &&& out.mmap_plans@ == mmap_plans
    &&& out.parsed@ == parsed
    &&& out.discovered == discovered
    &&& out.resolved == resolved
    &&& out.reloc_plan@ == expected_reloc_writes(parsed, discovered.order@, resolved)
    &&& reloc_writes_sound(parsed, discovered.order@, resolved, out.reloc_plan@)
}

} // verus!
