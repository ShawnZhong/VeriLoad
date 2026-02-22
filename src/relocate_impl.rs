use crate::consts::*;
use crate::relocate_spec::*;
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

fn add_i64_or_zero_exec(base: u64, addend: i64) -> (r: u64)
    ensures
        r == add_i64_or_zero(base, addend),
{
    let sum = (base as i128) + (addend as i128);
    if sum >= 0 && sum <= u64::MAX as i128 {
        sum as u64
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
    let raw = (DYN_BASE_START as i128) + (pos as i128) * (DYN_BASE_STRIDE as i128);
    if raw >= 0 && raw <= u64::MAX as i128 {
        raw as u64
    } else {
        0
    }
}

fn object_base_from_exec(parsed: &Vec<ParsedObject>, order: &Vec<usize>, obj_idx: usize, scan: usize) -> (r: u64)
    ensures
        r == object_base_from(parsed@, order@, obj_idx as int, scan as nat),
    decreases if scan < order.len() { order.len() - scan } else { 0 },
{
    if scan >= order.len() {
        0
    } else {
        let cur = order[scan];
        if cur == obj_idx && cur < parsed.len() {
            if parsed[cur].elf_type == ET_EXEC {
                0
            } else {
                dyn_base_for_pos_exec(scan)
            }
        } else {
            object_base_from_exec(parsed, order, obj_idx, scan + 1)
        }
    }
}

fn object_base_exec(parsed: &Vec<ParsedObject>, order: &Vec<usize>, obj_idx: usize) -> (r: u64)
    ensures
        r == object_base(parsed@, order@, obj_idx as int),
{
    object_base_from_exec(parsed, order, obj_idx, 0)
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

fn rela_type_exec(r: &RelaEntry) -> (t: u32)
    ensures
        t == rela_type_of(*r),
{
    (r.info & 0xffff_ffff) as u32
}

fn rr_reloc_entry_exec(parsed: &Vec<ParsedObject>, rr: &ResolvedReloc) -> (out: Option<RelaEntry>)
    ensures
        out == rr_reloc_entry(parsed@, *rr),
{
    if rr.requester < parsed.len() {
        let req = rr.requester;
        if rr.is_jmprel {
            if rr.reloc_index < parsed[req].jmprels.len() {
                let src = &parsed[req].jmprels[rr.reloc_index];
                Some(RelaEntry { offset: src.offset, info: src.info, addend: src.addend })
            } else {
                None
            }
        } else if rr.reloc_index < parsed[req].relas.len() {
            let src = &parsed[req].relas[rr.reloc_index];
            Some(RelaEntry { offset: src.offset, info: src.info, addend: src.addend })
        } else {
            None
        }
    } else {
        None
    }
}

fn rr_provider_value_exec(parsed: &Vec<ParsedObject>, order: &Vec<usize>, rr: &ResolvedReloc) -> (v: u64)
    ensures
        v == rr_provider_value(parsed@, order@, *rr),
{
    match (rr.provider_object, rr.provider_symbol) {
        (Some(po), Some(ps)) => {
            if po < parsed.len() && ps < parsed[po].dynsyms.len() {
                let base = object_base_exec(parsed, order, po);
                add_u64_or_zero_exec(base, parsed[po].dynsyms[ps].st_value)
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn patch_u64_le(bytes: &mut Vec<u8>, off: usize, value: u64) {
    if off >= bytes.len() {
        return;
    }
    let end = off.checked_add(8);
    if end.is_none() || end.unwrap() > bytes.len() {
        return;
    }
    let mut k: usize = 0;
    while k < 8
        invariant
            k <= 8,
            off + 8 <= bytes.len(),
        decreases 8 - k,
    {
        bytes[off + k] = ((value >> (8 * k)) & 0xff) as u8;
        k = k + 1;
    }
}

fn apply_write_to_plans(plans: &mut Vec<MmapPlan>, write: &RelocWrite) {
    let mut i: usize = 0;
    while i < plans.len()
        invariant
            i <= plans.len(),
        decreases plans.len() - i,
    {
        let mut plan = plans[i].clone();
        if plan.object_name == write.object_name && write.write_addr >= plan.start {
            let delta = write.write_addr - plan.start;
            if delta <= usize::MAX as u64 {
                patch_u64_le(&mut plan.bytes, delta as usize, write.value);
            }
        }
        plans[i] = plan;
        i = i + 1;
    }
}

fn plan_ranges_overlap_exec(a: &MmapPlan, b: &MmapPlan) -> (r: bool)
    ensures
        r == plan_ranges_overlap(*a, *b),
{
    let a_lo = a.start as u128;
    let a_hi = a_lo + a.bytes.len() as u128;
    let b_lo = b.start as u128;
    let b_hi = b_lo + b.bytes.len() as u128;
    a_lo < b_hi && b_lo < a_hi
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

fn symbol_is_weak_undef(sym: &DynSymbol) -> bool {
    let bind = sym.st_info >> 4;
    bind == 2 && sym.st_shndx == 0
}

fn symbol_relocation_requires_provider(rel_type: u32, sym: &DynSymbol) -> bool {
    (rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT) && !symbol_is_weak_undef(sym)
}

pub fn relocate_stage(
    parsed: Vec<ParsedObject>,
    discovered: DiscoveryResult,
    resolved: ResolutionResult,
) -> (out: Result<LoaderOutput, LoaderError>)
    ensures
        out.is_ok() ==> relocate_stage_spec(parsed@, discovered, resolved, out.unwrap()),
{
    let mut constructors: Vec<InitCall> = Vec::new();
    let mut i: usize = discovered.order.len();
    while i > 0
        invariant
            i <= discovered.order.len(),
        decreases i,
    {
        let idx = i - 1;
        let obj_idx = discovered.order[idx];
        if obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        let base = object_base_exec(&parsed, &discovered.order, obj_idx);
        let mut j: usize = 0;
        while j < parsed[obj_idx].init_array.len()
            invariant
                j <= parsed@[obj_idx as int].init_array@.len(),
                obj_idx < parsed.len(),
            decreases parsed@[obj_idx as int].init_array@.len() - j,
        {
            let call = InitCall {
                object_name: parsed[obj_idx].input_name.clone(),
                pc: add_u64_or_zero_exec(base, parsed[obj_idx].init_array[j]),
            };
            constructors.push(call);
            j = j + 1;
        }
        i = idx;
    }
    let mut destructors: Vec<TermCall> = Vec::new();
    let mut t: usize = 0;
    while t < discovered.order.len()
        invariant
            t <= discovered.order.len(),
        decreases discovered.order.len() - t,
    {
        let obj_idx = discovered.order[t];
        if obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        let base = object_base_exec(&parsed, &discovered.order, obj_idx);
        let mut j: usize = parsed[obj_idx].fini_array.len();
        while j > 0
            invariant
                j <= parsed@[obj_idx as int].fini_array@.len(),
                t < discovered.order.len(),
                obj_idx == discovered.order@[t as int],
                obj_idx < parsed.len(),
            decreases j,
        {
            let idx = j - 1;
            let call = TermCall {
                object_name: parsed[obj_idx].input_name.clone(),
                pc: add_u64_or_zero_exec(base, parsed[obj_idx].fini_array[idx]),
            };
            destructors.push(call);
            j = idx;
        }
        t = t + 1;
    }
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

    let mut reloc_writes: Vec<RelocWrite> = Vec::new();
    let mut ro: usize = 0;
    while ro < discovered.order.len()
        invariant
            ro <= discovered.order.len(),
            forall|k: int|
                0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                    parsed@,
                    discovered.order@,
                    mmap_plans@[k],
                ),
            mmap_plans_non_overlapping(mmap_plans@),
            forall|k: int|
                0 <= k < reloc_writes@.len() ==> write_matches_supported_relocation(
                    parsed@,
                    discovered.order@,
                    resolved,
                    reloc_writes@[k],
                ),
        decreases discovered.order.len() - ro,
    {
        let obj_idx = discovered.order[ro];
        if obj_idx < parsed.len() {
            let base = object_base_exec(&parsed, &discovered.order, obj_idx);
            let mut ri: usize = 0;
            while ri < parsed[obj_idx].relas.len()
                invariant
                    ri <= parsed@[obj_idx as int].relas@.len(),
                    ro < discovered.order.len(),
                    obj_idx == discovered.order@[ro as int],
                    obj_idx < parsed.len(),
                    base == object_base(parsed@, discovered.order@, obj_idx as int),
                    forall|k: int|
                        0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                            parsed@,
                            discovered.order@,
                            mmap_plans@[k],
                        ),
                    mmap_plans_non_overlapping(mmap_plans@),
                    forall|k: int|
                        0 <= k < reloc_writes@.len() ==> write_matches_supported_relocation(
                            parsed@,
                            discovered.order@,
                            resolved,
                            reloc_writes@[k],
                        ),
                decreases parsed@[obj_idx as int].relas@.len() - ri,
            {
                let rel = &parsed[obj_idx].relas[ri];
                if rela_type_exec(rel) == R_X86_64_RELATIVE {
                    let write_addr = add_u64_or_zero_exec(base, rel.offset);
                    let write_value = add_i64_or_zero_exec(base, rel.addend);
                    let write = RelocWrite {
                        object_name: parsed[obj_idx].input_name.clone(),
                        write_addr,
                        value: write_value,
                        reloc_type: R_X86_64_RELATIVE,
                    };
                    let ghost old_writes = reloc_writes@;
                    reloc_writes.push(write);
                    proof {
                        assert forall|k: int|
                            0 <= k < reloc_writes@.len() implies write_matches_supported_relocation(
                                parsed@,
                                discovered.order@,
                                resolved,
                                reloc_writes@[k],
                            ) by {
                            if k < old_writes.len() {
                            } else {
                                assert(k == old_writes.len());
                                assert(0 <= (obj_idx as int) && (obj_idx as int) < parsed@.len());
                                assert(base == object_base(parsed@, discovered.order@, obj_idx as int));
                                assert(rela_type_of(parsed@[obj_idx as int].relas@[ri as int]) == R_X86_64_RELATIVE);
                                assert(parsed@[obj_idx as int].relas@[ri as int].offset == rel.offset);
                                assert(parsed@[obj_idx as int].relas@[ri as int].addend == rel.addend);
                                assert(reloc_writes@[k].write_addr == write_addr);
                                assert(reloc_writes@[k].value == write_value);
                                assert(reloc_writes@[k].reloc_type == R_X86_64_RELATIVE);
                                assert(reloc_writes@[k].write_addr == add_u64_or_zero(
                                    object_base(parsed@, discovered.order@, obj_idx as int),
                                    parsed@[obj_idx as int].relas@[ri as int].offset,
                                ));
                                assert(reloc_writes@[k].value == add_i64_or_zero(
                                    object_base(parsed@, discovered.order@, obj_idx as int),
                                    parsed@[obj_idx as int].relas@[ri as int].addend,
                                ));
                                assert(write_matches_r_x86_64_relative_entry(
                                    parsed@,
                                    discovered.order@,
                                    obj_idx as int,
                                    parsed@[obj_idx as int].relas@[ri as int],
                                    reloc_writes@[k],
                                ));
                                assert(write_matches_any_r_x86_64_relative(
                                    parsed@,
                                    discovered.order@,
                                    reloc_writes@[k],
                                )) by {
                                    let p = ro as int;
                                    let i0 = ri as int;
                                    assert(0 <= p < discovered.order@.len());
                                    assert((discovered.order@[p] as int) < parsed@.len());
                                    assert(discovered.order@[p] == obj_idx);
                                };
                                assert(write_matches_supported_relocation(
                                    parsed@,
                                    discovered.order@,
                                    resolved,
                                    reloc_writes@[k],
                                ));
                            }
                        };
                    }
                }
                ri = ri + 1;
            }

            let mut ji: usize = 0;
            while ji < parsed[obj_idx].jmprels.len()
                invariant
                    ji <= parsed@[obj_idx as int].jmprels@.len(),
                    ro < discovered.order.len(),
                    obj_idx == discovered.order@[ro as int],
                    obj_idx < parsed.len(),
                    base == object_base(parsed@, discovered.order@, obj_idx as int),
                    forall|k: int|
                        0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                            parsed@,
                            discovered.order@,
                            mmap_plans@[k],
                        ),
                    mmap_plans_non_overlapping(mmap_plans@),
                    forall|k: int|
                        0 <= k < reloc_writes@.len() ==> write_matches_supported_relocation(
                            parsed@,
                            discovered.order@,
                            resolved,
                            reloc_writes@[k],
                        ),
                decreases parsed@[obj_idx as int].jmprels@.len() - ji,
            {
                let rel = &parsed[obj_idx].jmprels[ji];
                if rela_type_exec(rel) == R_X86_64_RELATIVE {
                    let write_addr = add_u64_or_zero_exec(base, rel.offset);
                    let write_value = add_i64_or_zero_exec(base, rel.addend);
                    let write = RelocWrite {
                        object_name: parsed[obj_idx].input_name.clone(),
                        write_addr,
                        value: write_value,
                        reloc_type: R_X86_64_RELATIVE,
                    };
                    let ghost old_writes = reloc_writes@;
                    reloc_writes.push(write);
                    proof {
                        assert forall|k: int|
                            0 <= k < reloc_writes@.len() implies write_matches_supported_relocation(
                                parsed@,
                                discovered.order@,
                                resolved,
                                reloc_writes@[k],
                            ) by {
                            if k < old_writes.len() {
                            } else {
                                assert(k == old_writes.len());
                                assert(0 <= (obj_idx as int) && (obj_idx as int) < parsed@.len());
                                assert(base == object_base(parsed@, discovered.order@, obj_idx as int));
                                assert(rela_type_of(parsed@[obj_idx as int].jmprels@[ji as int]) == R_X86_64_RELATIVE);
                                assert(parsed@[obj_idx as int].jmprels@[ji as int].offset == rel.offset);
                                assert(parsed@[obj_idx as int].jmprels@[ji as int].addend == rel.addend);
                                assert(reloc_writes@[k].write_addr == write_addr);
                                assert(reloc_writes@[k].value == write_value);
                                assert(reloc_writes@[k].reloc_type == R_X86_64_RELATIVE);
                                assert(reloc_writes@[k].write_addr == add_u64_or_zero(
                                    object_base(parsed@, discovered.order@, obj_idx as int),
                                    parsed@[obj_idx as int].jmprels@[ji as int].offset,
                                ));
                                assert(reloc_writes@[k].value == add_i64_or_zero(
                                    object_base(parsed@, discovered.order@, obj_idx as int),
                                    parsed@[obj_idx as int].jmprels@[ji as int].addend,
                                ));
                                assert(write_matches_r_x86_64_relative_entry(
                                    parsed@,
                                    discovered.order@,
                                    obj_idx as int,
                                    parsed@[obj_idx as int].jmprels@[ji as int],
                                    reloc_writes@[k],
                                ));
                                assert(write_matches_any_r_x86_64_relative(
                                    parsed@,
                                    discovered.order@,
                                    reloc_writes@[k],
                                )) by {
                                    let p = ro as int;
                                    let i0 = ji as int;
                                    assert(0 <= p < discovered.order@.len());
                                    assert((discovered.order@[p] as int) < parsed@.len());
                                    assert(discovered.order@[p] == obj_idx);
                                };
                                assert(write_matches_supported_relocation(
                                    parsed@,
                                    discovered.order@,
                                    resolved,
                                    reloc_writes@[k],
                                ));
                            }
                        };
                    }
                }
                ji = ji + 1;
            }
        } else {
            return Err(LoaderError {});
        }
        ro = ro + 1;
    }

    let mut rr_i: usize = 0;
    while rr_i < resolved.resolved_relocs.len()
        invariant
            rr_i <= resolved.resolved_relocs.len(),
            forall|k: int|
                0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                    parsed@,
                    discovered.order@,
                    mmap_plans@[k],
                ),
            mmap_plans_non_overlapping(mmap_plans@),
            forall|k: int|
                0 <= k < reloc_writes@.len() ==> write_matches_supported_relocation(
                    parsed@,
                    discovered.order@,
                    resolved,
                    reloc_writes@[k],
                ),
        decreases resolved.resolved_relocs.len() - rr_i,
    {
        let rr = &resolved.resolved_relocs[rr_i];
        let rel_opt = rr_reloc_entry_exec(&parsed, rr);
        match rel_opt {
            Some(rel) => {
                let rel_type = rela_type_exec(&rel);
                if rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT {
                    let req_idx = rr.requester;
                    if req_idx >= parsed.len() {
                        return Err(LoaderError {});
                    }
                    if rr.sym_index == 0 || rr.sym_index >= parsed[req_idx].dynsyms.len() {
                        return Err(LoaderError {});
                    }
                    let provider_required = symbol_relocation_requires_provider(
                        rel_type,
                        &parsed[req_idx].dynsyms[rr.sym_index],
                    );
                    match (rr.provider_object, rr.provider_symbol) {
                        (Some(po), Some(ps)) => {
                            if po >= parsed.len() || ps >= parsed[po].dynsyms.len() {
                                return Err(LoaderError {});
                            }
                        }
                        _ => {
                            if provider_required {
                                return Err(LoaderError {});
                            }
                        }
                    }
                    let req_base = object_base_exec(&parsed, &discovered.order, req_idx);
                    let value = rr_provider_value_exec(&parsed, &discovered.order, rr);
                    let write = RelocWrite {
                        object_name: parsed[req_idx].input_name.clone(),
                        write_addr: add_u64_or_zero_exec(req_base, rel.offset),
                        value,
                        reloc_type: rel_type,
                    };
                    let ghost old_writes = reloc_writes@;
                    reloc_writes.push(write);
                    proof {
                        assert forall|k: int|
                            0 <= k < reloc_writes@.len() implies write_matches_supported_relocation(
                                parsed@,
                                discovered.order@,
                                resolved,
                                reloc_writes@[k],
                            ) by {
                            if k < old_writes.len() {
                            } else {
                                assert(k == old_writes.len());
                                assert(write_matches_resolved_symbol_reloc(
                                    parsed@,
                                    discovered.order@,
                                    resolved.resolved_relocs@[rr_i as int],
                                    reloc_writes@[k],
                                ));
                                assert(write_matches_supported_relocation(
                                    parsed@,
                                    discovered.order@,
                                    resolved,
                                    reloc_writes@[k],
                                )) by {
                                    let i0 = rr_i as int;
                                    assert(0 <= i0 < resolved.resolved_relocs@.len());
                                };
                            }
                        };
                    }
                }
            }
            None => return Err(LoaderError {}),
        }
        rr_i = rr_i + 1;
    }

    let entry_pc = if parsed.len() == 0 {
        0
    } else {
        let main_base = object_base_exec(&parsed, &discovered.order, 0);
        add_u64_or_zero_exec(main_base, parsed[0].entry)
    };

    proof {
        assert(forall|k: int|
            0 <= k < mmap_plans@.len() ==> mmap_plan_sound(
                parsed@,
                discovered.order@,
                mmap_plans@[k],
            ));
        assert(mmap_plans_non_overlapping(mmap_plans@));
        assert(forall|k: int|
            0 <= k < reloc_writes@.len() ==> write_matches_supported_relocation(
                parsed@,
                discovered.order@,
                resolved,
                reloc_writes@[k],
            ));
        if parsed@.len() == 0 {
            assert(entry_pc == expected_entry_pc(parsed@, discovered.order@));
        } else {
            assert(parsed.len() > 0);
            assert(parsed@.len() > 0);
            assert(entry_pc == add_u64_or_zero(object_base(parsed@, discovered.order@, 0), parsed@[0].entry));
            assert(entry_pc == expected_entry_pc(parsed@, discovered.order@));
        }
    }

    Ok(LoaderOutput {
        entry_pc,
        constructors,
        destructors,
        mmap_plans,
        reloc_writes,
        parsed,
        discovered,
        resolved,
    })
}

} // verus!
