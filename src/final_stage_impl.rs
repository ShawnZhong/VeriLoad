use crate::consts::*;
use crate::final_stage_spec::*;
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

pub fn final_stage(plan: RelocateApplyOutput) -> (out: Result<LoaderOutput, LoaderError>)
    requires
        forall|i: int|
            0 <= i < plan.mmap_plans@.len() ==> mmap_plan_sound(
                plan.parsed@,
                plan.discovered.order@,
                plan.mmap_plans@[i],
            ),
        mmap_plans_non_overlapping(plan.mmap_plans@),
        reloc_writes_sound(plan.parsed@, plan.discovered.order@, plan.resolved, plan.reloc_writes@),
    ensures
        out.is_ok() ==> final_stage_spec(plan, out.unwrap()),
{
    let RelocateApplyOutput {
        mmap_plans,
        reloc_writes,
        parsed,
        discovered,
        resolved,
    } = plan;

    let mut constructors: Vec<InitCall> = Vec::new();
    let mut pos: usize = discovered.order.len();
    while pos > 0
        invariant
            pos <= discovered.order.len(),
            forall|k: int| 0 <= k < constructors@.len() ==> init_call_sound(
                parsed@,
                discovered.order@,
                constructors@[k],
            ),
        decreases pos,
    {
        let obj_pos = pos - 1;
        let obj_idx = discovered.order[obj_pos];
        if obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        let base = object_base_exec(&parsed, &discovered.order, obj_idx);
        let mut j: usize = 0;
        while j < parsed[obj_idx].init_array.len()
            invariant
                j <= parsed@[obj_idx as int].init_array@.len(),
                obj_pos < discovered.order.len(),
                obj_idx == discovered.order@[obj_pos as int],
                obj_idx < parsed.len(),
                base == object_base(parsed@, discovered.order@, obj_idx as int),
                forall|k: int| 0 <= k < constructors@.len() ==> init_call_sound(
                    parsed@,
                    discovered.order@,
                    constructors@[k],
                ),
            decreases parsed@[obj_idx as int].init_array@.len() - j,
        {
            let call = InitCall {
                object_name: clone_u8_vec(&parsed[obj_idx].input_name),
                pc: add_u64_or_zero_exec(base, parsed[obj_idx].init_array[j]),
            };
            let ghost old_calls = constructors@;
            constructors.push(call);
            proof {
                assert forall|k: int|
                    0 <= k < constructors@.len() implies init_call_sound(
                        parsed@,
                        discovered.order@,
                        constructors@[k],
                    ) by {
                    if k < old_calls.len() {
                    } else {
                        assert(k == old_calls.len());
                        let p = obj_pos as int;
                        let i0 = j as int;
                        assert(0 <= p < discovered.order@.len());
                        assert(discovered.order@[p] == obj_idx);
                        assert((discovered.order@[p] as int) < parsed@.len());
                        assert(0 <= i0 < parsed@[obj_idx as int].init_array@.len());
                        assert(constructors@[k].pc == add_u64_or_zero(
                            object_base(parsed@, discovered.order@, obj_idx as int),
                            parsed@[obj_idx as int].init_array@[i0],
                        ));
                        assert(init_call_sound(parsed@, discovered.order@, constructors@[k]));
                    }
                };
            }
            j = j + 1;
        }
        pos = obj_pos;
    }

    let mut destructors: Vec<TermCall> = Vec::new();
    let mut pos2: usize = 0;
    while pos2 < discovered.order.len()
        invariant
            pos2 <= discovered.order.len(),
            forall|k: int| 0 <= k < destructors@.len() ==> term_call_sound(
                parsed@,
                discovered.order@,
                destructors@[k],
            ),
        decreases discovered.order.len() - pos2,
    {
        let obj_idx = discovered.order[pos2];
        if obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        let base = object_base_exec(&parsed, &discovered.order, obj_idx);
        let mut j: usize = parsed[obj_idx].fini_array.len();
        while j > 0
            invariant
                j <= parsed@[obj_idx as int].fini_array@.len(),
                pos2 < discovered.order.len(),
                obj_idx == discovered.order@[pos2 as int],
                obj_idx < parsed.len(),
                base == object_base(parsed@, discovered.order@, obj_idx as int),
                forall|k: int| 0 <= k < destructors@.len() ==> term_call_sound(
                    parsed@,
                    discovered.order@,
                    destructors@[k],
                ),
            decreases j,
        {
            let idx = j - 1;
            let call = TermCall {
                object_name: clone_u8_vec(&parsed[obj_idx].input_name),
                pc: add_u64_or_zero_exec(base, parsed[obj_idx].fini_array[idx]),
            };
            let ghost old_calls = destructors@;
            destructors.push(call);
            proof {
                assert forall|k: int|
                    0 <= k < destructors@.len() implies term_call_sound(
                        parsed@,
                        discovered.order@,
                        destructors@[k],
                    ) by {
                    if k < old_calls.len() {
                    } else {
                        assert(k == old_calls.len());
                        let p = pos2 as int;
                        let i0 = idx as int;
                        assert(0 <= p < discovered.order@.len());
                        assert(discovered.order@[p] == obj_idx);
                        assert((discovered.order@[p] as int) < parsed@.len());
                        assert(0 <= i0 < parsed@[obj_idx as int].fini_array@.len());
                        assert(destructors@[k].pc == add_u64_or_zero(
                            object_base(parsed@, discovered.order@, obj_idx as int),
                            parsed@[obj_idx as int].fini_array@[i0],
                        ));
                        assert(term_call_sound(parsed@, discovered.order@, destructors@[k]));
                    }
                };
            }
            j = idx;
        }
        pos2 = pos2 + 1;
    }

    let entry_pc = if parsed.len() == 0 {
        0
    } else {
        let main_base = object_base_exec(&parsed, &discovered.order, 0);
        add_u64_or_zero_exec(main_base, parsed[0].entry)
    };

    let out_plan = LoaderOutput {
        entry_pc,
        constructors,
        destructors,
        mmap_plans,
        reloc_writes,
        parsed,
        discovered,
        resolved,
    };

    proof {
        assert(forall|i: int| 0 <= i < out_plan.constructors@.len() ==> init_call_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.constructors@[i],
        ));
        assert(forall|i: int| 0 <= i < out_plan.destructors@.len() ==> term_call_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.destructors@[i],
        ));
        assert(forall|i: int|
            0 <= i < out_plan.mmap_plans@.len() ==> mmap_plan_sound(
                out_plan.parsed@,
                out_plan.discovered.order@,
                out_plan.mmap_plans@[i],
            ));
        assert(mmap_plans_non_overlapping(out_plan.mmap_plans@));
        assert(reloc_writes_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.resolved,
            out_plan.reloc_writes@,
        ));
        if out_plan.parsed@.len() == 0 {
            assert(out_plan.entry_pc == expected_entry_pc(out_plan.parsed@, out_plan.discovered.order@));
        } else {
            assert(out_plan.parsed@.len() > 0);
            assert(out_plan.entry_pc == add_u64_or_zero(
                object_base(out_plan.parsed@, out_plan.discovered.order@, 0),
                out_plan.parsed@[0].entry,
            ));
            assert(out_plan.entry_pc == expected_entry_pc(out_plan.parsed@, out_plan.discovered.order@));
        }
    }

    Ok(out_plan)
}

} // verus!
