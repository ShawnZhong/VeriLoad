use crate::consts::*;
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

pub open spec fn seg_end_or_zero(vaddr: u64, memsz: u64) -> u64 {
    if vaddr <= u64::MAX - memsz {
        ((vaddr as int) + (memsz as int)) as u64
    } else {
        0
    }
}

pub open spec fn page_floor_u64(addr: u64) -> u64 {
    ((addr as int) - ((addr as int) % (PAGE_SIZE as int))) as u64
}

pub open spec fn page_ceil_u64(addr: u64) -> u64 {
    let rem = addr % PAGE_SIZE;
    if rem == 0 {
        addr
    } else {
        let raw = (addr as int) + (PAGE_SIZE as int) - (rem as int);
        if 0 <= raw && raw <= u64::MAX as int {
            raw as u64
        } else {
            0
        }
    }
}

pub open spec fn rounded_seg_start(base: u64, vaddr: u64) -> u64 {
    add_u64_or_zero(base, page_floor_u64(vaddr))
}

pub open spec fn rounded_seg_len(vaddr: u64, memsz: u64) -> nat {
    let lo = page_floor_u64(vaddr);
    let hi = page_ceil_u64(seg_end_or_zero(vaddr, memsz));
    if hi >= lo && hi - lo <= usize::MAX as u64 {
        (hi - lo) as nat
    } else {
        0
    }
}

pub open spec fn rela_type_of(r: RelaEntry) -> u32 {
    (r.info & 0xffff_ffff) as u32
}

pub open spec fn prot_of_flags(flags: u32) -> ProtFlags {
    ProtFlags {
        read: flags & PF_R == PF_R,
        write: flags & PF_W == PF_W,
        execute: flags & PF_X == PF_X,
    }
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

pub open spec fn segment_byte_at(obj: ParsedObject, ph: ProgramHeader, i: int) -> u8 {
    if 0 <= i < ph.p_filesz as int {
        let off = (ph.p_offset as int) + i;
        if 0 <= off < obj.file_bytes@.len() {
            obj.file_bytes@[off]
        } else {
            0
        }
    } else {
        0
    }
}

pub open spec fn segment_image(obj: ParsedObject, ph: ProgramHeader) -> Seq<u8> {
    if ph.p_memsz <= usize::MAX as u64 {
        Seq::new(ph.p_memsz as nat, |i: int| segment_byte_at(obj, ph, i))
    } else {
        Seq::empty()
    }
}

pub open spec fn mmap_plan_for_segment(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    obj_pos: int,
    ph_idx: int,
    plan: MmapPlan,
) -> bool {
    let obj_idx = order[obj_pos] as int;
    let ph = parsed[obj_idx].phdrs@[ph_idx];
    &&& 0 <= obj_pos < order.len()
    &&& 0 <= obj_idx < parsed.len()
    &&& 0 <= ph_idx < parsed[obj_idx].phdrs@.len()
    &&& ph.p_type == PT_LOAD
    &&& plan.object_name == parsed[obj_idx].input_name
    &&& plan.prot == prot_of_flags(ph.p_flags)
    &&& plan.start == rounded_seg_start(base_for_load_pos(parsed, order, obj_pos), ph.p_vaddr)
    &&& plan.bytes@.len() == rounded_seg_len(ph.p_vaddr, ph.p_memsz)
    &&& plan.start % PAGE_SIZE == 0
}

pub open spec fn mmap_plan_sound(parsed: Seq<ParsedObject>, order: Seq<usize>, plan: MmapPlan) -> bool {
    exists|obj_pos: int, ph_idx: int| mmap_plan_for_segment(parsed, order, obj_pos, ph_idx, plan)
}

pub open spec fn ranges_overlap_values(a_start: u64, a_len: nat, b_start: u64, b_len: nat) -> bool {
    let a_lo = a_start as int;
    let a_hi = a_lo + a_len as int;
    let b_lo = b_start as int;
    let b_hi = b_lo + b_len as int;
    a_lo < b_hi && b_lo < a_hi
}

pub open spec fn plan_ranges_overlap(a: MmapPlan, b: MmapPlan) -> bool {
    ranges_overlap_values(a.start, a.bytes@.len(), b.start, b.bytes@.len())
}

pub open spec fn same_plan_layout(a: MmapPlan, b: MmapPlan) -> bool {
    &&& a.object_name == b.object_name
    &&& a.start == b.start
    &&& a.prot == b.prot
    &&& a.bytes@.len() == b.bytes@.len()
}

pub open spec fn same_mmap_layout(old_plans: Seq<MmapPlan>, new_plans: Seq<MmapPlan>) -> bool {
    &&& old_plans.len() == new_plans.len()
    &&& forall|i: int| 0 <= i < old_plans.len() ==> same_plan_layout(old_plans[i], new_plans[i])
}

pub open spec fn mmap_plans_non_overlapping(plans: Seq<MmapPlan>) -> bool {
    forall|i: int, j: int|
        0 <= i < plans.len() && 0 <= j < plans.len() && i != j ==> !plan_ranges_overlap(plans[i], plans[j])
}

pub open spec fn count_load_phdrs_from(phdrs: Seq<ProgramHeader>, i: nat) -> nat
    decreases phdrs.len() - i,
{
    if i >= phdrs.len() {
        0
    } else {
        (if phdrs[i as int].p_type == PT_LOAD {
            1nat
        } else {
            0nat
        }) + count_load_phdrs_from(phdrs, (i + 1) as nat)
    }
}

pub open spec fn count_load_segments_from(parsed: Seq<ParsedObject>, order: Seq<usize>, pos: nat) -> nat
    decreases order.len() - pos,
{
    if pos >= order.len() {
        0
    } else {
        let obj_idx = order[pos as int] as int;
        (if 0 <= obj_idx < parsed.len() {
            count_load_phdrs_from(parsed[obj_idx].phdrs@, 0)
        } else {
            0
        }) + count_load_segments_from(parsed, order, (pos + 1) as nat)
    }
}

pub open spec fn count_load_segments(parsed: Seq<ParsedObject>, order: Seq<usize>) -> nat {
    count_load_segments_from(parsed, order, 0)
}

pub open spec fn reverse_order_index(order: Seq<usize>, p: int) -> int
    recommends
        0 <= p < order.len(),
{
    order.len() - 1 - p
}

pub open spec fn init_call_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    call: InitCall,
) -> bool {
    exists|p: int, i: int| {
        &&& 0 <= p < order.len()
        &&& (order[reverse_order_index(order, p)] as int) < parsed.len()
        &&& 0 <= i < parsed[order[reverse_order_index(order, p)] as int].init_array@.len()
        &&& call.pc == add_u64_or_zero(
            object_base(parsed, order, order[reverse_order_index(order, p)] as int),
            parsed[order[reverse_order_index(order, p)] as int].init_array@[i],
        )
    }
}

pub open spec fn term_call_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    call: TermCall,
) -> bool {
    exists|p: int, i: int| {
        &&& 0 <= p < order.len()
        &&& (order[p] as int) < parsed.len()
        &&& 0 <= i < parsed[order[p] as int].fini_array@.len()
        &&& call.pc == add_u64_or_zero(
            object_base(parsed, order, order[p] as int),
            parsed[order[p] as int].fini_array@[i],
        )
    }
}

pub open spec fn count_init_calls_from(parsed: Seq<ParsedObject>, order: Seq<usize>, pos: nat) -> nat
    decreases order.len() - pos,
{
    if pos >= order.len() {
        0
    } else {
        let obj_idx = order[pos as int] as int;
        (if 0 <= obj_idx < parsed.len() {
            parsed[obj_idx].init_array@.len()
        } else {
            0
        }) + count_init_calls_from(parsed, order, (pos + 1) as nat)
    }
}

pub open spec fn count_term_calls_from(parsed: Seq<ParsedObject>, order: Seq<usize>, pos: nat) -> nat
    decreases order.len() - pos,
{
    if pos >= order.len() {
        0
    } else {
        let obj_idx = order[pos as int] as int;
        (if 0 <= obj_idx < parsed.len() {
            parsed[obj_idx].fini_array@.len()
        } else {
            0
        }) + count_term_calls_from(parsed, order, (pos + 1) as nat)
    }
}

pub open spec fn count_init_calls(parsed: Seq<ParsedObject>, order: Seq<usize>) -> nat {
    count_init_calls_from(parsed, order, 0)
}

pub open spec fn count_term_calls(parsed: Seq<ParsedObject>, order: Seq<usize>) -> nat {
    count_term_calls_from(parsed, order, 0)
}

pub open spec fn expected_entry_pc(parsed: Seq<ParsedObject>, load_order: Seq<usize>) -> u64 {
    if parsed.len() == 0 {
        0
    } else {
        add_u64_or_zero(object_base(parsed, load_order, 0), parsed[0].entry)
    }
}

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

pub open spec fn count_relative_in_table_from(tab: Seq<RelaEntry>, i: nat) -> nat
    decreases tab.len() - i,
{
    if i >= tab.len() {
        0
    } else {
        (if rela_type_of(tab[i as int]) == R_X86_64_RELATIVE {
            1nat
        } else {
            0nat
        }) + count_relative_in_table_from(tab, (i + 1) as nat)
    }
}

pub open spec fn count_relative_obj(parsed: Seq<ParsedObject>, obj_idx: int) -> nat {
    if 0 <= obj_idx < parsed.len() {
        count_relative_in_table_from(parsed[obj_idx].relas@, 0) + count_relative_in_table_from(
            parsed[obj_idx].jmprels@,
            0,
        )
    } else {
        0
    }
}

pub open spec fn count_relative_relocs_from(parsed: Seq<ParsedObject>, order: Seq<usize>, pos: nat) -> nat
    decreases order.len() - pos,
{
    if pos >= order.len() {
        0
    } else {
        count_relative_obj(parsed, order[pos as int] as int) + count_relative_relocs_from(
            parsed,
            order,
            (pos + 1) as nat,
        )
    }
}

pub open spec fn count_relative_relocs(parsed: Seq<ParsedObject>, order: Seq<usize>) -> nat {
    count_relative_relocs_from(parsed, order, 0)
}

pub open spec fn count_resolved_jump_slots_from(parsed: Seq<ParsedObject>, rrs: Seq<ResolvedReloc>, i: nat) -> nat
    decreases rrs.len() - i,
{
    if i >= rrs.len() {
        0
    } else {
        (if match rr_reloc_entry(parsed, rrs[i as int]) {
            Some(rel) => rela_type_of(rel) == R_X86_64_JUMP_SLOT,
            None => false,
        } {
            1nat
        } else {
            0nat
        }) + count_resolved_jump_slots_from(parsed, rrs, (i + 1) as nat)
    }
}

pub open spec fn count_resolved_jump_slots(parsed: Seq<ParsedObject>, rrs: Seq<ResolvedReloc>) -> nat {
    count_resolved_jump_slots_from(parsed, rrs, 0)
}

pub open spec fn count_resolved_glob_dat_from(parsed: Seq<ParsedObject>, rrs: Seq<ResolvedReloc>, i: nat) -> nat
    decreases rrs.len() - i,
{
    if i >= rrs.len() {
        0
    } else {
        (if match rr_reloc_entry(parsed, rrs[i as int]) {
            Some(rel) => rela_type_of(rel) == R_X86_64_GLOB_DAT,
            None => false,
        } {
            1nat
        } else {
            0nat
        }) + count_resolved_glob_dat_from(parsed, rrs, (i + 1) as nat)
    }
}

pub open spec fn count_resolved_glob_dat(parsed: Seq<ParsedObject>, rrs: Seq<ResolvedReloc>) -> nat {
    count_resolved_glob_dat_from(parsed, rrs, 0)
}

pub open spec fn relocate_stage_spec(
    parsed: Seq<ParsedObject>,
    discovered: DiscoveryResult,
    resolved: ResolutionResult,
    out: LoaderOutput,
) -> bool {
    &&& out.entry_pc == expected_entry_pc(parsed, discovered.order@)
    &&& forall|i: int|
        0 <= i < out.mmap_plans@.len() ==> mmap_plan_sound(parsed, discovered.order@, out.mmap_plans@[i])
    &&& mmap_plans_non_overlapping(out.mmap_plans@)
    &&& forall|i: int|
        0 <= i < out.reloc_writes@.len() ==> write_matches_supported_relocation(
            parsed,
            discovered.order@,
            resolved,
            out.reloc_writes@[i],
        )
}

} // verus!
