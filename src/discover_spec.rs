use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn cstr_eq_from(a: Seq<u8>, ai: nat, b: Seq<u8>, bi: nat) -> bool
    decreases a.len() - ai,
{
    if ai >= a.len() || bi >= b.len() {
        false
    } else {
        let av = a[ai as int];
        let bv = b[bi as int];
        if av == 0 || bv == 0 {
            av == 0 && bv == 0
        } else {
            av == bv && cstr_eq_from(a, (ai + 1) as nat, b, (bi + 1) as nat)
        }
    }
}

pub open spec fn dep_edge(parsed: Seq<ParsedObject>, from: int, to: int) -> bool {
    &&& 0 <= from < parsed.len()
    &&& 0 <= to < parsed.len()
    &&& exists|k: int|
        0 <= k < parsed[from].needed_offsets@.len() && dep_target_matches(
            parsed,
            from,
            to,
            parsed[from].needed_offsets@[k] as nat,
        )
}

pub open spec fn dep_target_matches(parsed: Seq<ParsedObject>, from: int, to: int, need_off: nat) -> bool
    recommends
        0 <= from < parsed.len(),
        0 <= to < parsed.len(),
{
    match parsed[to].soname_offset {
        Some(soname_off) => cstr_eq_from(parsed[from].dynstr@, need_off, parsed[to].dynstr@, soname_off as nat),
        None => cstr_eq_from(parsed[from].dynstr@, need_off, parsed[to].input_name@.push(0u8), 0),
    }
}

pub open spec fn in_order_int(order: Seq<usize>, idx: int) -> bool {
    &&& 0 <= idx
    &&& exists|i: int| 0 <= i < order.len() && order[i] as int == idx
}

pub open spec fn valid_object_indices(order: Seq<usize>, n: nat) -> bool {
    forall|i: int| 0 <= i < order.len() ==> (order[i] as nat) < n
}

pub open spec fn unique_indices(order: Seq<usize>) -> bool {
    forall|i: int, j: int| 0 <= i < order.len() && 0 <= j < order.len() && i != j ==> order[i]
        != order[j]
}

pub open spec fn direct_dep_closure(parsed: Seq<ParsedObject>, order: Seq<usize>) -> bool {
    forall|p: int, v: int|
        0 <= p < parsed.len() && p < order.len() && 0 <= v < parsed.len() && dep_edge(parsed, order[p] as int, v) ==> in_order_int(
            order,
            v,
        )
}

pub open spec fn has_parent_edge(parsed: Seq<ParsedObject>, order: Seq<usize>, p: int) -> bool {
    exists|q: int| 0 <= q < p && dep_edge(parsed, order[q] as int, order[p] as int)
}

pub open spec fn non_root_has_parent_edge(parsed: Seq<ParsedObject>, order: Seq<usize>) -> bool {
    forall|p: int| 0 < p < order.len() ==> has_parent_edge(parsed, order, p)
}

pub open spec fn cycle_handling_policy(order: Seq<usize>) -> bool {
    unique_indices(order)
}

pub open spec fn reverse_of(a: Seq<usize>, b: Seq<usize>) -> bool {
    &&& a.len() == b.len()
    &&& forall|i: int| 0 <= i < a.len() ==> b[i] == a[a.len() - 1 - i]
}

pub open spec fn discover_stage_spec(parsed: Seq<ParsedObject>, out: DiscoveryResult) -> bool {
    &&& valid_object_indices(out.order@, parsed.len())
    &&& cycle_handling_policy(out.order@)
    &&& (parsed.len() == 0 ==> out.order@.len() == 0)
    &&& (parsed.len() > 0 ==> out.order@.len() > 0 && out.order@[0] == 0)
    &&& direct_dep_closure(parsed, out.order@)
    &&& non_root_has_parent_edge(parsed, out.order@)
}

} // verus!
