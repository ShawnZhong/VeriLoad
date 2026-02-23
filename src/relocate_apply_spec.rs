use crate::consts::*;
use crate::mmap_plan_spec::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn rr_reloc_entry(parsed: Seq<ParsedObject>, rr: ResolvedReloc) -> Option<RelaEntry> {
    let req = rr.requester as int;
    if 0 <= req < parsed.len() {
        if rr.is_jmprel {
            if (rr.reloc_index as int) < parsed[req].jmprels@.len() {
                Some(parsed[req].jmprels@[rr.reloc_index as int])
            } else {
                None
            }
        } else if (rr.reloc_index as int) < parsed[req].relas@.len() {
            Some(parsed[req].relas@[rr.reloc_index as int])
        } else {
            None
        }
    } else {
        None
    }
}

pub open spec fn rr_provider_value(parsed: Seq<ParsedObject>, order: Seq<usize>, rr: ResolvedReloc) -> u64 {
    match (rr.provider_object, rr.provider_symbol) {
        (Some(po), Some(ps)) => {
            if (po as int) < parsed.len() && (ps as int) < parsed[po as int].dynsyms@.len() {
                add_u64_or_zero(object_base(parsed, order, po as int), parsed[po as int].dynsyms@[ps as int].st_value)
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub open spec fn write_matches_r_x86_64_relative_entry(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_idx: int,
    rela: RelaEntry,
    w: RelocWrite,
) -> bool {
    &&& 0 <= obj_idx < parsed.len()
    &&& rela_type_of(rela) == R_X86_64_RELATIVE
    &&& w.write_addr == add_u64_or_zero(object_base(parsed, order, obj_idx), rela.offset)
    &&& w.value == add_i64_or_zero(object_base(parsed, order, obj_idx), rela.addend)
    &&& w.reloc_type == R_X86_64_RELATIVE
}

pub open spec fn write_matches_any_r_x86_64_relative(parsed: Seq<ParsedObject>, order: Seq<usize>, w: RelocWrite) -> bool {
    exists|p: int| {
        &&& 0 <= p < order.len()
        &&& (order[p] as int) < parsed.len()
        &&& (
            exists|i: int| {
                0 <= i < parsed[order[p] as int].relas@.len() && write_matches_r_x86_64_relative_entry(
                    parsed,
                    order,
                    order[p] as int,
                    parsed[order[p] as int].relas@[i],
                    w,
                )
            }
        )
        || (
            exists|i: int| {
                0 <= i < parsed[order[p] as int].jmprels@.len() && write_matches_r_x86_64_relative_entry(
                    parsed,
                    order,
                    order[p] as int,
                    parsed[order[p] as int].jmprels@[i],
                    w,
                )
            }
        )
    }
}

pub open spec fn write_matches_resolved_symbol_reloc(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    rr: ResolvedReloc,
    w: RelocWrite,
) -> bool {
    write_matches_r_x86_64_jump_slot_from_rr(parsed, order, rr, w)
        || write_matches_r_x86_64_glob_dat_from_rr(parsed, order, rr, w)
}

pub open spec fn write_matches_r_x86_64_jump_slot_from_rr(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    rr: ResolvedReloc,
    w: RelocWrite,
) -> bool {
    let req = rr.requester as int;
    &&& 0 <= req < parsed.len()
    &&& match rr_reloc_entry(parsed, rr) {
        Some(rel) => {
            &&& rela_type_of(rel) == R_X86_64_JUMP_SLOT
            &&& w.write_addr == add_u64_or_zero(object_base(parsed, order, req), rel.offset)
            &&& w.value == rr_provider_value(parsed, order, rr)
            &&& w.reloc_type == R_X86_64_JUMP_SLOT
        }
        None => false,
    }
}

pub open spec fn write_matches_r_x86_64_glob_dat_from_rr(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    rr: ResolvedReloc,
    w: RelocWrite,
) -> bool {
    let req = rr.requester as int;
    &&& 0 <= req < parsed.len()
    &&& match rr_reloc_entry(parsed, rr) {
        Some(rel) => {
            &&& rela_type_of(rel) == R_X86_64_GLOB_DAT
            &&& w.write_addr == add_u64_or_zero(object_base(parsed, order, req), rel.offset)
            &&& w.value == rr_provider_value(parsed, order, rr)
            &&& w.reloc_type == R_X86_64_GLOB_DAT
        }
        None => false,
    }
}

pub open spec fn write_matches_supported_relocation(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    resolved: ResolutionResult,
    w: RelocWrite,
) -> bool {
    write_matches_any_r_x86_64_relative(parsed, order, w)
        || exists|i: int|
            0 <= i < resolved.resolved_relocs@.len() && write_matches_r_x86_64_jump_slot_from_rr(
                parsed,
                order,
                resolved.resolved_relocs@[i],
                w,
            )
        || exists|i: int|
            0 <= i < resolved.resolved_relocs@.len() && write_matches_r_x86_64_glob_dat_from_rr(
                parsed,
                order,
                resolved.resolved_relocs@[i],
                w,
            )
}

pub open spec fn reloc_writes_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    resolved: ResolutionResult,
    reloc_writes: Seq<RelocWrite>,
) -> bool {
    forall|i: int| 0 <= i < reloc_writes.len() ==> write_matches_supported_relocation(
        parsed,
        order,
        resolved,
        reloc_writes[i],
    )
}

pub open spec fn u64_le_byte(value: u64, idx: int) -> u8
    recommends
        0 <= idx < 8,
{
    let shift = (idx * 8) as u64;
    ((value >> shift) & 0xffu64) as u8
}

pub open spec fn patch_u64_le_bytes(bytes: Seq<u8>, off: nat, value: u64) -> Seq<u8> {
    if off + 8 <= bytes.len() {
        Seq::new(
            bytes.len(),
            |i: int|
                if off <= i && i < off + 8 {
                    u64_le_byte(value, i - off)
                } else {
                    bytes[i]
                },
        )
    } else {
        bytes
    }
}

pub open spec fn apply_write_to_plan_bytes(
    plan: MmapPlan,
    cur_bytes: Seq<u8>,
    write: RelocWrite,
) -> Seq<u8> {
    if write.write_addr >= plan.start
        && write.write_addr - plan.start <= usize::MAX as u64
    {
        patch_u64_le_bytes(cur_bytes, (write.write_addr - plan.start) as nat, write.value)
    } else {
        cur_bytes
    }
}

pub open spec fn initial_plan_bytes(plans: Seq<MmapPlan>) -> Seq<Seq<u8>> {
    Seq::new(plans.len(), |i: int| plans[i].bytes@)
}

pub open spec fn apply_write_to_all_plan_bytes(
    plans: Seq<MmapPlan>,
    cur: Seq<Seq<u8>>,
    write: RelocWrite,
) -> Seq<Seq<u8>>
    recommends
        cur.len() == plans.len(),
{
    Seq::new(
        plans.len(),
        |i: int| apply_write_to_plan_bytes(plans[i], cur[i], write),
    )
}

pub open spec fn apply_write_to_plans_current_bytes(
    plans: Seq<MmapPlan>,
    write: RelocWrite,
) -> Seq<Seq<u8>> {
    apply_write_to_all_plan_bytes(plans, initial_plan_bytes(plans), write)
}

pub open spec fn apply_reloc_writes_prefix_bytes(
    plans: Seq<MmapPlan>,
    writes: Seq<RelocWrite>,
    n: nat,
) -> Seq<Seq<u8>>
    decreases n,
{
    if n == 0 {
        initial_plan_bytes(plans)
    } else {
        apply_write_to_all_plan_bytes(
            plans,
            apply_reloc_writes_prefix_bytes(plans, writes, (n - 1) as nat),
            writes[(n - 1) as int],
        )
    }
}

pub open spec fn apply_reloc_writes_bytes(plans: Seq<MmapPlan>, writes: Seq<RelocWrite>) -> Seq<Seq<u8>> {
    apply_reloc_writes_prefix_bytes(plans, writes, writes.len() as nat)
}

pub open spec fn relocate_apply_stage_spec(
    in_plan: RelocatePlanOutput,
    out_plan: RelocateApplyOutput,
) -> bool {
    &&& out_plan.mmap_plans@.len() == in_plan.mmap_plans@.len()
    &&& forall|i: int|
        0 <= i < out_plan.mmap_plans@.len() ==> out_plan.mmap_plans@[i].bytes@ == apply_reloc_writes_bytes(
            in_plan.mmap_plans@,
            in_plan.reloc_plan@,
        )[i]
    &&& same_mmap_layout(in_plan.mmap_plans@, out_plan.mmap_plans@)
    &&& out_plan.reloc_writes@ == in_plan.reloc_plan@
    &&& out_plan.parsed@ == in_plan.parsed@
    &&& out_plan.discovered == in_plan.discovered
    &&& out_plan.resolved == in_plan.resolved
    &&& forall|i: int|
        0 <= i < out_plan.mmap_plans@.len() ==> mmap_plan_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.mmap_plans@[i],
        )
    &&& mmap_plans_non_overlapping(out_plan.mmap_plans@)
    &&& reloc_writes_sound(
        out_plan.parsed@,
        out_plan.discovered.order@,
        out_plan.resolved,
        out_plan.reloc_writes@,
    )
}

} // verus!
