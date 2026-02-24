use crate::consts::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

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

pub open spec fn prot_of_flags(flags: u32) -> ProtFlags {
    ProtFlags {
        read: flags & PF_R == PF_R,
        write: flags & PF_W == PF_W,
        execute: flags & PF_X == PF_X,
    }
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
    &&& plan.object_name@ == parsed[obj_idx].input_name@
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
    &&& a.object_name@ == b.object_name@
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

pub open spec fn mmap_plan_stage_spec(
    parsed: Seq<ParsedObject>,
    discovered: DiscoveryResult,
    mmap_plans: Seq<MmapPlan>,
) -> bool {
    &&& forall|i: int| 0 <= i < mmap_plans.len() ==> mmap_plan_sound(parsed, discovered.order@, mmap_plans[i])
    &&& mmap_plans_non_overlapping(mmap_plans)
}

} // verus!
