use crate::consts::*;
use crate::mmap_plan_spec::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

fn add_u64_or_zero_exec(a: u64, b: u64) -> (r: u64)
    ensures
        r == add_u64_or_zero(a, b),
{
    if a <= u64::MAX - b {
        a + b
    } else {
        0
    }
}

fn seg_end_or_zero_exec(vaddr: u64, memsz: u64) -> (r: u64)
    ensures
        r == seg_end_or_zero(vaddr, memsz),
{
    if vaddr <= u64::MAX - memsz {
        vaddr + memsz
    } else {
        0
    }
}

fn page_floor_u64_exec(addr: u64) -> (r: u64)
    ensures
        r == page_floor_u64(addr),
{
    addr - (addr % PAGE_SIZE)
}

fn page_ceil_u64_exec(addr: u64) -> (r: u64)
    ensures
        r == page_ceil_u64(addr),
{
    let rem = addr % PAGE_SIZE;
    if rem == 0 {
        addr
    } else {
        add_u64_or_zero_exec(addr, PAGE_SIZE - rem)
    }
}

fn rounded_seg_start_exec(base: u64, vaddr: u64) -> (r: u64)
    ensures
        r == rounded_seg_start(base, vaddr),
{
    add_u64_or_zero_exec(base, page_floor_u64_exec(vaddr))
}

fn rounded_seg_len_exec(vaddr: u64, memsz: u64) -> (n: usize)
    ensures
        n as nat == rounded_seg_len(vaddr, memsz),
{
    let lo = page_floor_u64_exec(vaddr);
    let hi = page_ceil_u64_exec(seg_end_or_zero_exec(vaddr, memsz));
    if hi >= lo && hi - lo <= usize::MAX as u64 {
        (hi - lo) as usize
    } else {
        0
    }
}

fn dyn_base_for_pos_exec(pos: usize) -> (r: u64)
    ensures
        r == dyn_base_for_pos(pos as int),
{
    let mul = (pos as i128).checked_mul(DYN_BASE_STRIDE as i128);
    if mul.is_none() {
        0
    } else {
        let raw_opt = (DYN_BASE_START as i128).checked_add(mul.unwrap());
        if raw_opt.is_none() {
            0
        } else {
            let raw = raw_opt.unwrap();
            if raw >= 0 && raw <= u64::MAX as i128 {
                raw as u64
            } else {
                0
            }
        }
    }
}

fn prot_of_flags_exec(flags: u32) -> (p: ProtFlags)
    ensures
        p == prot_of_flags(flags),
{
    ProtFlags {
        read: flags & PF_R == PF_R,
        write: flags & PF_W == PF_W,
        execute: flags & PF_X == PF_X,
    }
}

fn mem_len_exec(memsz: u64) -> (n: usize)
    ensures
        n as nat == if memsz <= usize::MAX as u64 {
            memsz as nat
        } else {
            0
        },
{
    if memsz <= usize::MAX as u64 {
        memsz as usize
    } else {
        0
    }
}

fn segment_bytes_exec(obj: &ParsedObject, ph: &ProgramHeader) -> (out: Vec<u8>)
    ensures
        out@.len() == if ph.p_memsz <= usize::MAX as u64 {
            ph.p_memsz as nat
        } else {
            0
        },
{
    let len = mem_len_exec(ph.p_memsz);
    let mut out: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < len
        invariant
            i <= len,
            out@.len() == i,
        decreases len - i,
    {
        let mut b: u8 = 0;
        if (i as u64) < ph.p_filesz {
            let off_u64_opt = ph.p_offset.checked_add(i as u64);
            if off_u64_opt.is_some() {
                let off_u64 = off_u64_opt.unwrap();
                if off_u64 <= usize::MAX as u64 {
                    let off = off_u64 as usize;
                    if off < obj.file_bytes.len() {
                        b = obj.file_bytes[off];
                    }
                }
            }
        }
        out.push(b);
        i = i + 1;
    }
    out
}

fn segment_mmap_bytes_exec(obj: &ParsedObject, ph: &ProgramHeader) -> (out: Vec<u8>)
    ensures
        out@.len() == rounded_seg_len(ph.p_vaddr, ph.p_memsz),
{
    let lo = page_floor_u64_exec(ph.p_vaddr);
    let off_u64 = ph.p_vaddr - lo;
    let off = if off_u64 <= usize::MAX as u64 {
        off_u64 as usize
    } else {
        0
    };
    let len = rounded_seg_len_exec(ph.p_vaddr, ph.p_memsz);
    let seg = segment_bytes_exec(obj, ph);

    let mut out: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < off && out.len() < len
        invariant
            i <= off,
            out@.len() <= len,
            out@.len() == i,
        decreases off - i,
    {
        out.push(0u8);
        i = i + 1;
    }

    i = 0;
    while i < seg.len() && out.len() < len
        invariant
            i <= seg.len(),
            out@.len() <= len,
            out@.len() <= off + i,
        decreases seg.len() - i,
    {
        out.push(seg[i]);
        i = i + 1;
    }

    while out.len() < len
        invariant
            out@.len() <= len,
        decreases len - out.len(),
    {
        out.push(0u8);
    }
    out
}

fn plan_ranges_overlap_values_exec(a: &MmapPlan, b_start: u64, b_len: usize) -> (r: bool)
    ensures
        r == ranges_overlap_values(a.start, a.bytes@.len(), b_start, b_len as nat),
{
    let a_lo = a.start as u128;
    let a_hi = a_lo + a.bytes.len() as u128;
    let b_lo = b_start as u128;
    let b_hi = b_lo + b_len as u128;
    a_lo < b_hi && b_lo < a_hi
}

pub fn mmap_plan_stage(
    parsed: &Vec<ParsedObject>,
    discovered: &DiscoveryResult,
) -> (out: Result<Vec<MmapPlan>, LoaderError>)
    ensures
        out.is_ok() ==> mmap_plan_stage_spec(parsed@, *discovered, out.unwrap()@),
{
    let mut mmap_plans: Vec<MmapPlan> = Vec::new();
    let mut oi: usize = 0;
    while oi < discovered.order.len()
        invariant
            oi <= discovered.order.len(),
            forall|k: int|
                0 <= k < mmap_plans@.len() ==> mmap_plan_sound(parsed@, discovered.order@, mmap_plans@[k]),
            mmap_plans_non_overlapping(mmap_plans@),
        decreases discovered.order.len() - oi,
    {
        let obj_idx = discovered.order[oi];
        if obj_idx < parsed.len() {
            let base = if parsed[obj_idx].elf_type == ET_EXEC {
                0
            } else {
                dyn_base_for_pos_exec(oi)
            };
            proof {
                assert(base == base_for_load_pos(parsed@, discovered.order@, oi as int));
            }
            let mut pi: usize = 0;
            while pi < parsed[obj_idx].phdrs.len()
                invariant
                    pi <= parsed@[obj_idx as int].phdrs@.len(),
                    oi < discovered.order.len(),
                    obj_idx == discovered.order@[oi as int],
                    obj_idx < parsed.len(),
                    base == base_for_load_pos(parsed@, discovered.order@, oi as int),
                    forall|k: int|
                        0 <= k < mmap_plans@.len() ==> mmap_plan_sound(parsed@, discovered.order@, mmap_plans@[k]),
                    mmap_plans_non_overlapping(mmap_plans@),
                decreases parsed@[obj_idx as int].phdrs@.len() - pi,
            {
                let ph = &parsed[obj_idx].phdrs[pi];
                if ph.p_type == PT_LOAD {
                    let cand_start = rounded_seg_start_exec(base, ph.p_vaddr);
                    let bytes = segment_mmap_bytes_exec(&parsed[obj_idx], ph);
                    let cand_len = bytes.len();
                    let cand = MmapPlan {
                        object_name: parsed[obj_idx].input_name.clone(),
                        start: cand_start,
                        bytes,
                        prot: prot_of_flags_exec(ph.p_flags),
                    };
                    let mut collides: bool = false;
                    let mut ci: usize = 0;
                    while ci < mmap_plans.len()
                        invariant
                            ci <= mmap_plans.len(),
                            !collides ==> forall|k: int|
                                0 <= k < ci ==> !ranges_overlap_values(
                                    mmap_plans@[k].start,
                                    mmap_plans@[k].bytes@.len(),
                                    cand_start,
                                    cand_len as nat,
                                ),
                        decreases mmap_plans.len() - ci,
                    {
                        let hit = plan_ranges_overlap_values_exec(&mmap_plans[ci], cand_start, cand_len);
                        if hit {
                            collides = true;
                            ci = mmap_plans.len();
                        } else {
                            ci = ci + 1;
                        }
                    }
                    if !collides {
                        let ghost old_plans = mmap_plans@;
                        mmap_plans.push(cand);
                        proof {
                            assert(ci == old_plans.len());
                            let old_last = old_plans.len() as int;
                            assert(mmap_plans@[old_last].object_name == parsed@[obj_idx as int].input_name);
                            assert(mmap_plans@[old_last].prot == prot_of_flags(parsed@[obj_idx as int].phdrs@[pi as int].p_flags));
                            assert(mmap_plans@[old_last].start == cand_start);
                            assert(mmap_plans@[old_last].bytes@.len() == cand_len as nat);
                            assert forall|k: int| 0 <= k < old_plans.len() implies !ranges_overlap_values(
                                old_plans[k].start,
                                old_plans[k].bytes@.len(),
                                cand_start,
                                cand_len as nat,
                            ) by {
                                assert(!ranges_overlap_values(
                                    old_plans[k].start,
                                    old_plans[k].bytes@.len(),
                                    cand_start,
                                    cand_len as nat,
                                ));
                            };
                            assert forall|k: int| 0 <= k < old_plans.len() implies !plan_ranges_overlap(
                                old_plans[k],
                                mmap_plans@[old_last],
                            ) by {
                                assert(!ranges_overlap_values(
                                    old_plans[k].start,
                                    old_plans[k].bytes@.len(),
                                    mmap_plans@[old_last].start,
                                    mmap_plans@[old_last].bytes@.len(),
                                ));
                                assert(!plan_ranges_overlap(old_plans[k], mmap_plans@[old_last]));
                            };
                            assert forall|k: int|
                                0 <= k < mmap_plans@.len() implies mmap_plan_sound(
                                    parsed@,
                                    discovered.order@,
                                    mmap_plans@[k],
                                ) by {
                                if k < old_plans.len() {
                                } else {
                                    assert(k == old_plans.len());
                                    assert(mmap_plan_sound(parsed@, discovered.order@, mmap_plans@[k])) by {
                                        let p = oi as int;
                                        let h = pi as int;
                                        let obj = obj_idx as int;
                                        assert(0 <= p < discovered.order@.len());
                                        assert(discovered.order@[p] == obj_idx);
                                        assert(0 <= obj < parsed@.len());
                                        assert(obj == discovered.order@[p] as int);
                                        assert(0 <= h < parsed@[obj].phdrs@.len());
                                        assert(parsed@[obj].phdrs@[h].p_type == PT_LOAD);
                                        assert(mmap_plans@[k].object_name == parsed@[obj].input_name);
                                        assert(mmap_plans@[k].prot == prot_of_flags(parsed@[obj].phdrs@[h].p_flags));
                                        assert(mmap_plans@[k].start == rounded_seg_start(
                                            base_for_load_pos(parsed@, discovered.order@, p),
                                            parsed@[obj].phdrs@[h].p_vaddr,
                                        ));
                                        assert(mmap_plans@[k].bytes@.len() == rounded_seg_len(
                                            parsed@[obj].phdrs@[h].p_vaddr,
                                            parsed@[obj].phdrs@[h].p_memsz,
                                        ));
                                        assert(mmap_plans@[k].start % PAGE_SIZE == 0);
                                        assert(mmap_plan_for_segment(parsed@, discovered.order@, p, h, mmap_plans@[k]));
                                    };
                                }
                            };
                            assert(mmap_plans_non_overlapping(mmap_plans@)) by {
                                assert forall|i0: int, j0: int|
                                    0 <= i0 < mmap_plans@.len() && 0 <= j0 < mmap_plans@.len() && i0 != j0 implies !plan_ranges_overlap(
                                        mmap_plans@[i0],
                                        mmap_plans@[j0],
                                    ) by {
                                    if i0 < old_plans.len() && j0 < old_plans.len() {
                                        assert(!plan_ranges_overlap(old_plans[i0], old_plans[j0]));
                                    } else if i0 < old_plans.len() {
                                        assert(j0 == old_plans.len());
                                        assert(!plan_ranges_overlap(old_plans[i0], mmap_plans@[j0]));
                                    } else {
                                        assert(i0 == old_plans.len());
                                        assert(j0 < old_plans.len());
                                        assert(!plan_ranges_overlap(old_plans[j0], mmap_plans@[i0]));
                                    }
                                };
                            };
                        }
                    }
                }
                pi = pi + 1;
            }
        } else {
            return Err(LoaderError {});
        }
        oi = oi + 1;
    }
    proof {
        assert(forall|k: int|
            0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                parsed@,
                discovered.order@,
                mmap_plans@[k],
            ));
        assert(mmap_plans_non_overlapping(mmap_plans@));
    }
    Ok(mmap_plans)
}

} // verus!
