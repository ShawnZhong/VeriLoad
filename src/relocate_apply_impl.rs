use crate::mmap_plan_spec::*;
use crate::relocate_apply_spec::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

proof fn same_plan_layout_refl(a: MmapPlan)
    ensures
        same_plan_layout(a, a),
{
    assert(a.object_name@ == a.object_name@);
    assert(a.start == a.start);
    assert(a.prot == a.prot);
    assert(a.bytes@.len() == a.bytes@.len());
}

proof fn same_plan_layout_transitive(a: MmapPlan, b: MmapPlan, c: MmapPlan)
    requires
        same_plan_layout(a, b),
        same_plan_layout(b, c),
    ensures
        same_plan_layout(a, c),
{
    assert(a.object_name@ == b.object_name@);
    assert(b.object_name@ == c.object_name@);
    assert(a.object_name@ == c.object_name@);

    assert(a.start == b.start);
    assert(b.start == c.start);
    assert(a.start == c.start);

    assert(a.prot == b.prot);
    assert(b.prot == c.prot);
    assert(a.prot == c.prot);

    assert(a.bytes@.len() == b.bytes@.len());
    assert(b.bytes@.len() == c.bytes@.len());
    assert(a.bytes@.len() == c.bytes@.len());
}

proof fn same_mmap_layout_update_index(old_plans: Seq<MmapPlan>, idx: int, new_plan: MmapPlan)
    requires
        0 <= idx < old_plans.len(),
        same_plan_layout(old_plans[idx], new_plan),
    ensures
        same_mmap_layout(old_plans, old_plans.update(idx, new_plan)),
{
    assert(old_plans.update(idx, new_plan).len() == old_plans.len());
    assert forall|k: int| 0 <= k < old_plans.len() implies same_plan_layout(
        old_plans[k],
        old_plans.update(idx, new_plan)[k],
    ) by {
        if k == idx {
            assert(old_plans.update(idx, new_plan)[k] == new_plan);
        } else {
            assert(old_plans.update(idx, new_plan)[k] == old_plans[k]);
            same_plan_layout_refl(old_plans[k]);
        }
    };
}

proof fn same_mmap_layout_transitive(a: Seq<MmapPlan>, b: Seq<MmapPlan>, c: Seq<MmapPlan>)
    requires
        same_mmap_layout(a, b),
        same_mmap_layout(b, c),
    ensures
        same_mmap_layout(a, c),
{
    assert(a.len() == b.len());
    assert(b.len() == c.len());
    assert(a.len() == c.len());
    assert forall|i: int| 0 <= i < a.len() implies same_plan_layout(a[i], c[i]) by {
        same_plan_layout_transitive(a[i], b[i], c[i]);
    };
}

proof fn same_plan_layout_preserves_mmap_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    old_plan: MmapPlan,
    new_plan: MmapPlan,
)
    requires
        mmap_plan_sound(parsed, order, old_plan),
        same_plan_layout(old_plan, new_plan),
    ensures
        mmap_plan_sound(parsed, order, new_plan),
{
    assert forall|obj_pos: int, ph_idx: int|
        mmap_plan_for_segment(parsed, order, obj_pos, ph_idx, old_plan) implies mmap_plan_for_segment(
            parsed,
            order,
            obj_pos,
            ph_idx,
            new_plan,
        ) by {
        assert(old_plan.object_name@ == new_plan.object_name@);
        assert(old_plan.start == new_plan.start);
        assert(old_plan.prot == new_plan.prot);
        assert(old_plan.bytes@.len() == new_plan.bytes@.len());
    };
    assert(exists|obj_pos: int, ph_idx: int| mmap_plan_for_segment(
        parsed,
        order,
        obj_pos,
        ph_idx,
        old_plan,
    ));
    assert(exists|obj_pos: int, ph_idx: int| mmap_plan_for_segment(
        parsed,
        order,
        obj_pos,
        ph_idx,
        new_plan,
    ));
    assert(mmap_plan_sound(parsed, order, new_plan));
}

proof fn same_mmap_layout_preserves_mmap_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    old_plans: Seq<MmapPlan>,
    new_plans: Seq<MmapPlan>,
)
    requires
        same_mmap_layout(old_plans, new_plans),
        forall|i: int| 0 <= i < old_plans.len() ==> mmap_plan_sound(parsed, order, old_plans[i]),
    ensures
        forall|i: int| 0 <= i < new_plans.len() ==> mmap_plan_sound(parsed, order, new_plans[i]),
{
    assert(old_plans.len() == new_plans.len());
    assert forall|i: int| 0 <= i < new_plans.len() implies mmap_plan_sound(parsed, order, new_plans[i]) by {
        assert(0 <= i < old_plans.len());
        same_plan_layout_preserves_mmap_sound(parsed, order, old_plans[i], new_plans[i]);
    };
}

proof fn same_mmap_layout_preserves_non_overlapping(old_plans: Seq<MmapPlan>, new_plans: Seq<MmapPlan>)
    requires
        same_mmap_layout(old_plans, new_plans),
        mmap_plans_non_overlapping(old_plans),
    ensures
        mmap_plans_non_overlapping(new_plans),
{
    assert(old_plans.len() == new_plans.len());
    assert forall|i: int, j: int|
        0 <= i < new_plans.len() && 0 <= j < new_plans.len() && i != j implies !plan_ranges_overlap(
            new_plans[i],
            new_plans[j],
        ) by {
        assert(0 <= i < old_plans.len());
        assert(0 <= j < old_plans.len());
        assert(!plan_ranges_overlap(old_plans[i], old_plans[j]));
        assert(new_plans[i].start == old_plans[i].start);
        assert(new_plans[i].bytes@.len() == old_plans[i].bytes@.len());
        assert(new_plans[j].start == old_plans[j].start);
        assert(new_plans[j].bytes@.len() == old_plans[j].bytes@.len());
        assert(plan_ranges_overlap(new_plans[i], new_plans[j]) == ranges_overlap_values(
            new_plans[i].start,
            new_plans[i].bytes@.len(),
            new_plans[j].start,
            new_plans[j].bytes@.len(),
        ));
        assert(plan_ranges_overlap(old_plans[i], old_plans[j]) == ranges_overlap_values(
            old_plans[i].start,
            old_plans[i].bytes@.len(),
            old_plans[j].start,
            old_plans[j].bytes@.len(),
        ));
        assert(ranges_overlap_values(
            new_plans[i].start,
            new_plans[i].bytes@.len(),
            new_plans[j].start,
            new_plans[j].bytes@.len(),
        ) == ranges_overlap_values(
            old_plans[i].start,
            old_plans[i].bytes@.len(),
            old_plans[j].start,
            old_plans[j].bytes@.len(),
        ));
    };
}

proof fn apply_write_to_plan_bytes_same_layout(a: MmapPlan, b: MmapPlan, cur: Seq<u8>, write: RelocWrite)
    requires
        same_plan_layout(a, b),
    ensures
        apply_write_to_plan_bytes(a, cur, write) == apply_write_to_plan_bytes(b, cur, write),
{
    assert(a.object_name@ == b.object_name@);
    assert(a.start == b.start);
}

proof fn patch_u64_le_bytes_preserves_len(bytes: Seq<u8>, off: nat, value: u64)
    ensures
        patch_u64_le_bytes(bytes, off, value).len() == bytes.len(),
{
}

proof fn apply_reloc_writes_prefix_bytes_len(plans: Seq<MmapPlan>, writes: Seq<RelocWrite>, n: nat)
    requires
        n <= writes.len(),
    ensures
        apply_reloc_writes_prefix_bytes(plans, writes, n).len() == plans.len(),
    decreases n,
{
    if n == 0 {
    } else {
        apply_reloc_writes_prefix_bytes_len(plans, writes, (n - 1) as nat);
    }
}

fn patch_u64_le(bytes: &mut Vec<u8>, off: usize, value: u64)
    ensures
        bytes@ == patch_u64_le_bytes(old(bytes)@, off as nat, value),
{
    if off > bytes.len() || bytes.len() - off < 8 {
        proof {
            assert(off as nat + 8 > old(bytes)@.len());
            assert(patch_u64_le_bytes(old(bytes)@, off as nat, value) == old(bytes)@);
        }
        return;
    }

    let mut k: usize = 0;
    while k < 8
        invariant
            k <= 8,
            off + 8 <= bytes.len(),
            bytes@.len() == old(bytes)@.len(),
            forall|j: int| 0 <= j < bytes@.len() ==> (
                if off as int <= j && j < (off as int + k as int) {
                    bytes@[j] == u64_le_byte(value, j - off as int)
                } else {
                    bytes@[j] == old(bytes)@[j]
                }
            ),
        decreases 8 - k,
    {
        let idx = off + k;
        let ghost before = bytes@;
        let shift = 8 * k;
        let b = ((value >> shift) & 0xff) as u8;
        bytes[idx] = b;
        proof {
            assert(b == u64_le_byte(value, k as int));
            assert(bytes@ == before.update(idx as int, b));
            assert forall|j: int| 0 <= j < bytes@.len() implies (
                if off as int <= j && j < (off as int + (k + 1) as int) {
                    bytes@[j] == u64_le_byte(value, j - off as int)
                } else {
                    bytes@[j] == old(bytes)@[j]
                }
            ) by {
                if j == idx as int {
                    assert(j - off as int == k as int);
                    assert(bytes@[j] == b);
                } else {
                    assert(bytes@[j] == before[j]);
                    if off as int <= j && j < (off as int + k as int) {
                        assert(before[j] == u64_le_byte(value, j - off as int));
                    } else {
                        assert(before[j] == old(bytes)@[j]);
                    }
                }
            };
        }
        k = k + 1;
    }

    proof {
        assert(off as nat + 8 <= old(bytes)@.len());
        patch_u64_le_bytes_preserves_len(old(bytes)@, off as nat, value);
        assert forall|j: int| 0 <= j < bytes@.len() implies bytes@[j] == patch_u64_le_bytes(
            old(bytes)@,
            off as nat,
            value,
        )[j] by {
            if off as int <= j && j < (off as int + 8) {
                assert(bytes@[j] == u64_le_byte(value, j - off as int));
            } else {
                assert(bytes@[j] == old(bytes)@[j]);
            }
        };
        assert(bytes@ == patch_u64_le_bytes(old(bytes)@, off as nat, value));
    }
}

fn apply_write_to_plans(plans: &mut Vec<MmapPlan>, write: &RelocWrite)
    ensures
        same_mmap_layout(old(plans)@, plans@),
        forall|i: int|
            0 <= i < plans@.len() ==> plans@[i].bytes@ == apply_write_to_plan_bytes(
                plans@[i],
                old(plans)@[i].bytes@,
                *write,
            ),
{
    let mut i: usize = 0;
    while i < plans.len()
        invariant
            i <= plans.len(),
            plans@.len() == old(plans)@.len(),
            same_mmap_layout(old(plans)@, plans@),
            forall|k: int|
                0 <= k < i ==> plans@[k].bytes@ == apply_write_to_plan_bytes(
                    plans@[k],
                    old(plans)@[k].bytes@,
                    *write,
                ),
            forall|k: int| i <= k < plans@.len() ==> plans@[k] == old(plans)@[k],
        decreases plans.len() - i,
    {
        let ghost before = plans@;
        let ghost before_i = before[i as int];
        let mut plan = plans.remove(i);
        proof {
            assert(plan == before_i);
        }
        let ghost bytes_before = plan.bytes@;
        let should_patch = write.write_addr >= plan.start
            && write.write_addr - plan.start <= usize::MAX as u64;
        let ghost should_patch_expr = write.write_addr >= plan.start
            && write.write_addr - plan.start <= usize::MAX as u64;
        proof {
            assert(plan.object_name@ == before_i.object_name@);
            assert(plan.start == before_i.start);
            assert(should_patch == should_patch_expr);
        }
        if should_patch {
            let delta = write.write_addr - plan.start;
            let ghost before_patch = plan.bytes@;
            patch_u64_le(&mut plan.bytes, delta as usize, write.value);
            proof {
                assert(before_patch == bytes_before);
                assert(plan.bytes@ == patch_u64_le_bytes(
                    bytes_before,
                    delta as nat,
                    write.value,
                ));
            }
        } else {
            proof {
                assert(plan.bytes@ == bytes_before);
            }
        }

        plans.insert(i, plan);

        proof {
            assert(before_i == old(plans)@[i as int]);
            assert(plans@[i as int] == plan);

            assert(plans@[i as int].object_name@ == before_i.object_name@);
            assert(plans@[i as int].start == before_i.start);
            assert(plans@[i as int].prot == before_i.prot);
            assert(bytes_before == before_i.bytes@);
            assert(plans@[i as int].bytes@.len() == before_i.bytes@.len()) by {
                if write.write_addr >= before_i.start {
                    let delta = write.write_addr - before_i.start;
                    if delta <= usize::MAX as u64 {
                        patch_u64_le_bytes_preserves_len(before_i.bytes@, delta as nat, write.value);
                    }
                }
            };
            assert(same_plan_layout(before_i, plans@[i as int]));

            assert(plans@[i as int].bytes@ == apply_write_to_plan_bytes(
                plan,
                before_i.bytes@,
                *write,
            )) by {
                if should_patch {
                    assert(should_patch_expr);
                    let delta = write.write_addr - plan.start;
                    assert(apply_write_to_plan_bytes(
                        plan,
                        before_i.bytes@,
                        *write,
                    ) == patch_u64_le_bytes(before_i.bytes@, delta as nat, write.value));
                    assert(plans@[i as int].bytes@ == patch_u64_le_bytes(
                        before_i.bytes@,
                        delta as nat,
                        write.value,
                    ));
                } else {
                    assert(!should_patch_expr);
                    assert(apply_write_to_plan_bytes(
                        plan,
                        before_i.bytes@,
                        *write,
                    ) == before_i.bytes@);
                    assert(plans@[i as int].bytes@ == bytes_before);
                    assert(plans@[i as int].bytes@ == before_i.bytes@);
                }
            };
            assert(apply_write_to_plan_bytes(
                plan,
                before_i.bytes@,
                *write,
            ) == apply_write_to_plan_bytes(
                plans@[i as int],
                before_i.bytes@,
                *write,
            ));
            assert(plans@[i as int].bytes@ == apply_write_to_plan_bytes(
                plans@[i as int],
                before_i.bytes@,
                *write,
            ));

            same_mmap_layout_update_index(before, i as int, plans@[i as int]);
            assert(plans@ == before.update(i as int, plans@[i as int]));
            same_mmap_layout_transitive(old(plans)@, before, plans@);

            assert forall|k: int| 0 <= k < i + 1 implies plans@[k].bytes@ == apply_write_to_plan_bytes(
                plans@[k],
                old(plans)@[k].bytes@,
                *write,
            ) by {
                if k < i as int {
                    assert(plans@[k] == before[k]);
                } else {
                    assert(k == i as int);
                    assert(before_i == old(plans)@[k]);
                    assert(plans@[k].bytes@ == apply_write_to_plan_bytes(
                        plans@[k],
                        before_i.bytes@,
                        *write,
                    ));
                }
            };

            assert forall|k: int| i + 1 <= k < plans@.len() implies plans@[k] == old(plans)@[k] by {
                assert(plans@[k] == before[k]);
                assert(before[k] == old(plans)@[k]);
            };
        }

        i = i + 1;
    }
}

pub fn relocate_apply_stage(plan: RelocatePlanOutput) -> (out: Result<RelocateApplyOutput, LoaderError>)
    requires
        forall|i: int|
            0 <= i < plan.mmap_plans@.len() ==> mmap_plan_sound(
                plan.parsed@,
                plan.discovered.order@,
                plan.mmap_plans@[i],
            ),
        mmap_plans_non_overlapping(plan.mmap_plans@),
        reloc_writes_sound(plan.parsed@, plan.discovered.order@, plan.resolved, plan.reloc_plan@),
    ensures
        out.is_ok() ==> relocate_apply_stage_spec(plan, out.unwrap()),
{
    let RelocatePlanOutput {
        mmap_plans: mut mmap_plans,
        reloc_plan,
        parsed,
        discovered,
        resolved,
    } = plan;
    let ghost in_mmap_plans = mmap_plans@;

    proof {
        apply_reloc_writes_prefix_bytes_len(in_mmap_plans, reloc_plan@, 0);
        assert forall|i: int|
            0 <= i < mmap_plans@.len() implies mmap_plans@[i].bytes@ == apply_reloc_writes_prefix_bytes(
                in_mmap_plans,
                reloc_plan@,
                0,
            )[i] by {
            assert(mmap_plans@[i] == in_mmap_plans[i]);
        };
    }

    let mut wi: usize = 0;
    while wi < reloc_plan.len()
        invariant
            wi <= reloc_plan@.len(),
            mmap_plans@.len() == in_mmap_plans.len(),
            same_mmap_layout(in_mmap_plans, mmap_plans@),
            apply_reloc_writes_prefix_bytes(in_mmap_plans, reloc_plan@, wi as nat).len() == in_mmap_plans.len(),
            forall|i: int|
                0 <= i < mmap_plans@.len() ==> mmap_plans@[i].bytes@ == apply_reloc_writes_prefix_bytes(
                    in_mmap_plans,
                    reloc_plan@,
                    wi as nat,
                )[i],
        decreases reloc_plan.len() - wi,
    {
        let ghost before_plans = mmap_plans@;
        apply_write_to_plans(&mut mmap_plans, &reloc_plan[wi]);
        proof {
            same_mmap_layout_transitive(in_mmap_plans, before_plans, mmap_plans@);
            apply_reloc_writes_prefix_bytes_len(in_mmap_plans, reloc_plan@, (wi + 1) as nat);

            assert forall|i: int|
                0 <= i < mmap_plans@.len() implies mmap_plans@[i].bytes@ == apply_reloc_writes_prefix_bytes(
                    in_mmap_plans,
                    reloc_plan@,
                    (wi + 1) as nat,
                )[i] by {
                assert(0 <= i < before_plans.len());
                assert(mmap_plans@[i].bytes@ == apply_write_to_plan_bytes(
                    mmap_plans@[i],
                    before_plans[i].bytes@,
                    reloc_plan@[wi as int],
                ));
                assert(before_plans[i].bytes@ == apply_reloc_writes_prefix_bytes(
                    in_mmap_plans,
                    reloc_plan@,
                    wi as nat,
                )[i]);
                apply_write_to_plan_bytes_same_layout(
                    mmap_plans@[i],
                    in_mmap_plans[i],
                    before_plans[i].bytes@,
                    reloc_plan@[wi as int],
                );
                assert(apply_write_to_plan_bytes(
                    mmap_plans@[i],
                    before_plans[i].bytes@,
                    reloc_plan@[wi as int],
                ) == apply_write_to_plan_bytes(
                    in_mmap_plans[i],
                    before_plans[i].bytes@,
                    reloc_plan@[wi as int],
                ));
                assert(apply_reloc_writes_prefix_bytes(
                    in_mmap_plans,
                    reloc_plan@,
                    (wi + 1) as nat,
                )[i] == apply_write_to_plan_bytes(
                    in_mmap_plans[i],
                    apply_reloc_writes_prefix_bytes(in_mmap_plans, reloc_plan@, wi as nat)[i],
                    reloc_plan@[wi as int],
                ));
            };
        }
        wi = wi + 1;
    }

    let out_plan = RelocateApplyOutput {
        mmap_plans,
        reloc_writes: reloc_plan,
        parsed,
        discovered,
        resolved,
    };

    proof {
        assert(same_mmap_layout(in_mmap_plans, out_plan.mmap_plans@));
        apply_reloc_writes_prefix_bytes_len(in_mmap_plans, out_plan.reloc_writes@, out_plan.reloc_writes@.len() as nat);
        assert forall|i: int|
            0 <= i < out_plan.mmap_plans@.len() implies out_plan.mmap_plans@[i].bytes@ == apply_reloc_writes_bytes(
                in_mmap_plans,
                out_plan.reloc_writes@,
            )[i] by {
            assert(out_plan.reloc_writes@ == reloc_plan@);
            assert(0 <= i < out_plan.mmap_plans@.len());
            assert(out_plan.mmap_plans@[i].bytes@ == apply_reloc_writes_prefix_bytes(
                in_mmap_plans,
                reloc_plan@,
                reloc_plan@.len() as nat,
            )[i]);
            assert(apply_reloc_writes_bytes(in_mmap_plans, out_plan.reloc_writes@) == apply_reloc_writes_prefix_bytes(
                in_mmap_plans,
                reloc_plan@,
                reloc_plan@.len() as nat,
            ));
        };
        assert(forall|i: int|
            0 <= i < in_mmap_plans.len() ==> mmap_plan_sound(parsed@, discovered.order@, in_mmap_plans[i]));
        same_mmap_layout_preserves_mmap_sound(
            parsed@,
            discovered.order@,
            in_mmap_plans,
            out_plan.mmap_plans@,
        );
        assert(mmap_plans_non_overlapping(in_mmap_plans));
        same_mmap_layout_preserves_non_overlapping(in_mmap_plans, out_plan.mmap_plans@);
        assert(reloc_writes_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.resolved,
            out_plan.reloc_writes@,
        ));
    }

    Ok(out_plan)
}

} // verus!
