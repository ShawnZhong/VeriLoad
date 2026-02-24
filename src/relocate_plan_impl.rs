use crate::consts::*;
use crate::mmap_plan_spec::*;
use crate::relocate_apply_spec::*;
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

fn symbol_is_weak_undef(sym: &DynSymbol) -> bool {
    let bind = sym.st_info >> 4;
    bind == 2 && sym.st_shndx == 0
}

fn symbol_relocation_requires_provider(rel_type: u32, sym: &DynSymbol) -> bool {
    (rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT) && !symbol_is_weak_undef(sym)
}

pub fn plan_relocate_stage(
    parsed: Vec<ParsedObject>,
    discovered: DiscoveryResult,
    resolved: ResolutionResult,
    mmap_plans: Vec<MmapPlan>,
) -> (out: Result<RelocatePlanOutput, LoaderError>)
    requires
        mmap_plan_stage_spec(parsed@, discovered, mmap_plans@),
    ensures
        out.is_ok() ==> plan_relocate_stage_spec(parsed@, discovered, resolved, mmap_plans@, out.unwrap()),
{
    // This stage only plans relocation writes.

    let mut reloc_writes: Vec<RelocWrite> = Vec::new();
    let ghost expected_total = expected_reloc_writes(parsed@, discovered.order@, resolved);
    let mut ro: usize = 0;
    while ro < discovered.order.len()
        invariant
            ro <= discovered.order.len(),
            reloc_writes@
                + expected_relative_writes_from_order(parsed@, discovered.order@, ro as nat)
                + expected_symbol_writes_from(parsed@, discovered.order@, resolved, 0) == expected_total,
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
                    reloc_writes@
                        + expected_relative_rela_writes_from(
                            parsed@,
                            discovered.order@,
                            obj_idx as int,
                            ri as nat,
                        )
                        + expected_relative_jmprel_writes_from(
                            parsed@,
                            discovered.order@,
                            obj_idx as int,
                            0,
                        )
                        + expected_relative_writes_from_order(
                            parsed@,
                            discovered.order@,
                            (ro + 1) as nat,
                        )
                        + expected_symbol_writes_from(parsed@, discovered.order@, resolved, 0)
                        == expected_total,
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
                        object_name: clone_u8_vec(&parsed[obj_idx].input_name),
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
                    reloc_writes@
                        + expected_relative_jmprel_writes_from(
                            parsed@,
                            discovered.order@,
                            obj_idx as int,
                            ji as nat,
                        )
                        + expected_relative_writes_from_order(
                            parsed@,
                            discovered.order@,
                            (ro + 1) as nat,
                        )
                        + expected_symbol_writes_from(parsed@, discovered.order@, resolved, 0)
                        == expected_total,
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
                        object_name: clone_u8_vec(&parsed[obj_idx].input_name),
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
            reloc_writes@ + expected_symbol_writes_from(
                parsed@,
                discovered.order@,
                resolved,
                rr_i as nat,
            ) == expected_total,
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
                        object_name: clone_u8_vec(&parsed[req_idx].input_name),
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
        assert(reloc_writes_sound(parsed@, discovered.order@, resolved, reloc_writes@));
    }

    Ok(RelocatePlanOutput {
        mmap_plans,
        reloc_plan: reloc_writes,
        parsed,
        discovered,
        resolved,
    })
}

} // verus!
