use crate::discover_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

fn cstr_eq_from_exec(a: &Vec<u8>, ai: usize, b: &Vec<u8>, bi: usize) -> (r: bool)
    ensures
        r == cstr_eq_from(a@, ai as nat, b@, bi as nat),
    decreases if ai < a.len() { a.len() - ai } else { 0 },
{
    if ai >= a.len() || bi >= b.len() {
        false
    } else {
        let av = a[ai];
        let bv = b[bi];
        if av == 0 || bv == 0 {
            av == 0 && bv == 0
        } else if av != bv {
            false
        } else {
            cstr_eq_from_exec(a, ai + 1, b, bi + 1)
        }
    }
}

fn contains_index(order: &Vec<usize>, idx: usize) -> (r: bool)
    ensures
        r == in_order_int(order@, idx as int),
{
    let mut i: usize = 0;
    while i < order.len()
        invariant
            i <= order.len(),
            forall|k: int| 0 <= k < i ==> order@[k] != idx,
        decreases order.len() - i,
    {
        if order[i] == idx {
            proof {
                assert(in_order_int(order@, idx as int));
            }
            return true;
        }
        i = i + 1;
    }
    proof {
        assert(!in_order_int(order@, idx as int));
    }
    false
}

fn depends_on(parsed: &Vec<ParsedObject>, from: usize, to: usize) -> (r: bool)
    requires
        from < parsed@.len(),
        to < parsed@.len(),
    ensures
        r == dep_edge(parsed@, from as int, to as int),
{
    proof {
        assert(from < parsed.len());
        assert(to < parsed.len());
    }

    let soname_opt = parsed[to].soname_offset;
    let mut input_name_cstr: Vec<u8> = Vec::new();
    if soname_opt.is_none() {
        input_name_cstr = clone_u8_vec(&parsed[to].input_name);
        input_name_cstr.push(0u8);
    }

    let mut k: usize = 0;
    while k < parsed[from].needed_offsets.len()
        invariant
            from < parsed.len(),
            to < parsed.len(),
            soname_opt == parsed@[to as int].soname_offset,
            soname_opt.is_none() ==> input_name_cstr@ == parsed@[to as int].input_name@.push(0u8),
            k <= parsed@[from as int].needed_offsets@.len(),
            forall|j: int|
                0 <= j < k ==> !dep_target_matches(
                    parsed@,
                    from as int,
                    to as int,
                    parsed@[from as int].needed_offsets@[j] as nat,
                ),
        decreases parsed@[from as int].needed_offsets@.len() - k,
    {
        let need_off = parsed[from].needed_offsets[k];
        if soname_opt.is_some() {
            let soname_off = soname_opt.unwrap();
            let eq = cstr_eq_from_exec(
                &parsed[from].dynstr,
                need_off as usize,
                &parsed[to].dynstr,
                soname_off as usize,
            );
            if eq {
                proof {
                    assert(dep_target_matches(parsed@, from as int, to as int, need_off as nat));
                    let k_int: int = k as int;
                    assert(0 <= k_int && k_int < parsed@[from as int].needed_offsets@.len());
                    assert(parsed@[from as int].needed_offsets@[k_int] == need_off);
                    assert(exists|j: int|
                        0 <= j < parsed@[from as int].needed_offsets@.len() && dep_target_matches(
                            parsed@,
                            from as int,
                            to as int,
                            parsed@[from as int].needed_offsets@[j] as nat,
                        )) by {
                        let j = k as int;
                        assert(0 <= j < parsed@[from as int].needed_offsets@.len());
                        assert(parsed@[from as int].needed_offsets@[j] as nat == need_off as nat);
                    };
                    assert(dep_edge(parsed@, from as int, to as int));
                }
                return true;
            }
            proof {
                assert(!dep_target_matches(parsed@, from as int, to as int, need_off as nat));
            }
        } else {
            let eq = cstr_eq_from_exec(
                &parsed[from].dynstr,
                need_off as usize,
                &input_name_cstr,
                0,
            );
            if eq {
                proof {
                    assert(dep_target_matches(parsed@, from as int, to as int, need_off as nat));
                    let k_int: int = k as int;
                    assert(0 <= k_int && k_int < parsed@[from as int].needed_offsets@.len());
                    assert(parsed@[from as int].needed_offsets@[k_int] == need_off);
                    assert(exists|j: int|
                        0 <= j < parsed@[from as int].needed_offsets@.len() && dep_target_matches(
                            parsed@,
                            from as int,
                            to as int,
                            parsed@[from as int].needed_offsets@[j] as nat,
                        )) by {
                        let j = k as int;
                        assert(0 <= j < parsed@[from as int].needed_offsets@.len());
                        assert(parsed@[from as int].needed_offsets@[j] as nat == need_off as nat);
                    };
                    assert(dep_edge(parsed@, from as int, to as int));
                }
                return true;
            }
            proof {
                assert(!dep_target_matches(parsed@, from as int, to as int, need_off as nat));
            }
        }
        k = k + 1;
    }

    proof {
        assert(k == parsed@[from as int].needed_offsets@.len());
        assert(!dep_edge(parsed@, from as int, to as int)) by {
            if dep_edge(parsed@, from as int, to as int) {
                let j = choose|j: int|
                    0 <= j < parsed@[from as int].needed_offsets@.len() && dep_target_matches(
                        parsed@,
                        from as int,
                        to as int,
                        parsed@[from as int].needed_offsets@[j] as nat,
                    );
                assert(j < k as int);
                assert(!dep_target_matches(
                    parsed@,
                    from as int,
                    to as int,
                    parsed@[from as int].needed_offsets@[j] as nat,
                ));
                assert(false);
            }
        };
    }
    false
}

fn has_needed_match(parsed: &Vec<ParsedObject>, from: usize, need_off: u32) -> (r: bool) {
    if from >= parsed.len() {
        return false;
    }

    let mut to: usize = 0;
    while to < parsed.len()
        invariant
            to <= parsed.len(),
            from < parsed.len(),
        decreases parsed.len() - to,
    {
        let soname_opt = parsed[to].soname_offset;
        if soname_opt.is_some() {
            let soname_off = soname_opt.unwrap();
            let eq = cstr_eq_from_exec(
                &parsed[from].dynstr,
                need_off as usize,
                &parsed[to].dynstr,
                soname_off as usize,
            );
            if eq {
                return true;
            }
        } else {
            let mut input_name_cstr = clone_u8_vec(&parsed[to].input_name);
            input_name_cstr.push(0u8);
            let eq = cstr_eq_from_exec(
                &parsed[from].dynstr,
                need_off as usize,
                &input_name_cstr,
                0,
            );
            if eq {
                return true;
            }
        }
        to = to + 1;
    }
    false
}

proof fn lemma_in_order_push_preserve(order: Seq<usize>, idx: int, x: usize)
    requires
        in_order_int(order, idx),
    ensures
        in_order_int(order.push(x), idx),
{
    let w = choose|i: int| 0 <= i < order.len() && order[i] as int == idx;
    assert(0 <= w < order.push(x).len());
    assert(order.push(x)[w] as int == idx);
}

proof fn lemma_in_order_push_new(order: Seq<usize>, x: usize)
    ensures
        in_order_int(order.push(x), x as int),
{
    let w: int = order.len() as int;
    assert(0 <= w < order.push(x).len());
    assert(order.push(x)[w] == x);
}

proof fn lemma_valid_indices_push(order: Seq<usize>, n: nat, x: usize)
    requires
        valid_object_indices(order, n),
        (x as nat) < n,
    ensures
        valid_object_indices(order.push(x), n),
{
    assert forall|i: int| 0 <= i < order.push(x).len() implies (order.push(x)[i] as nat) < n by {
        if i < order.len() {
        } else {
            assert(i == order.len());
            assert(order.push(x)[i] == x);
        }
    };
}

proof fn lemma_unique_indices_push(order: Seq<usize>, x: usize)
    requires
        unique_indices(order),
        !in_order_int(order, x as int),
    ensures
        unique_indices(order.push(x)),
{
    assert forall|i: int, j: int|
        0 <= i < order.push(x).len() && 0 <= j < order.push(x).len() && i != j implies order.push(
            x,
        )[i] != order.push(x)[j] by {
        if i < order.len() {
            if j < order.len() {
            } else {
                assert(j == order.len());
                assert(order.push(x)[j] == x);
                assert(order[i] != x) by {
                    if order[i] == x {
                        assert(in_order_int(order, x as int));
                    }
                };
            }
        } else {
            assert(i == order.len());
            if j < order.len() {
                assert(order.push(x)[i] == x);
                assert(x != order[j]) by {
                    if x == order[j] {
                        assert(in_order_int(order, x as int));
                    }
                };
            } else {
                assert(j == order.len());
                assert(false);
            }
        }
    };
}

proof fn lemma_non_root_parent_push(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    x: usize,
    parent_pos: int,
)
    requires
        non_root_has_parent_edge(parsed, order),
        0 <= parent_pos < order.len(),
        dep_edge(parsed, order[parent_pos] as int, x as int),
    ensures
        non_root_has_parent_edge(parsed, order.push(x)),
{
    assert forall|p: int| 0 < p < order.push(x).len() implies has_parent_edge(parsed, order.push(x), p) by {
        if p < order.len() {
            assert(0 < p < order.len());
            assert(has_parent_edge(parsed, order, p));
            let q = choose|q: int| 0 <= q < p && dep_edge(parsed, order[q] as int, order[p] as int);
            assert(0 <= q < p);
            assert(order.push(x)[q] == order[q]);
            assert(order.push(x)[p] == order[p]);
            assert(dep_edge(parsed, order.push(x)[q] as int, order.push(x)[p] as int));
            assert(has_parent_edge(parsed, order.push(x), p));
        } else {
            assert(p == order.len());
            let q = parent_pos;
            assert(0 <= q < p);
            assert(order.push(x)[q] == order[q]);
            assert(order.push(x)[p] == x);
            assert(dep_edge(parsed, order.push(x)[q] as int, order.push(x)[p] as int));
            assert(has_parent_edge(parsed, order.push(x), p));
        }
    };
}

pub fn discover_stage(parsed: &Vec<ParsedObject>) -> (out: Result<DiscoveryResult, LoaderError>)
    ensures
        out.is_ok() ==> discover_stage_spec(parsed@, out.unwrap()),
{
    let mut order: Vec<usize> = Vec::new();

    if parsed.len() == 0 {
        return Ok(DiscoveryResult { order: order });
    }

    order.push(0);

    let mut q: usize = 0;
    while q < parsed.len()
        invariant
            q <= parsed.len(),
            order@.len() > 0,
            order@[0] == 0,
            valid_object_indices(order@, parsed@.len()),
            unique_indices(order@),
            non_root_has_parent_edge(parsed@, order@),
            forall|p: int, v: int|
                0 <= p < q && p < order@.len() && 0 <= v < parsed.len() && dep_edge(
                    parsed@,
                    order@[p] as int,
                    v,
                ) ==> in_order_int(order@, v),
        decreases parsed.len() - q,
    {
        if q < order.len() {
            let cur = order[q];
            let mut cand: usize = 0;
            while cand < parsed.len()
                invariant
                    cand <= parsed.len(),
                    q < order@.len(),
                    cur == order@[q as int],
                    order@.len() > 0,
                    order@[0] == 0,
                    valid_object_indices(order@, parsed@.len()),
                    unique_indices(order@),
                    non_root_has_parent_edge(parsed@, order@),
                    forall|p: int, v: int|
                        0 <= p < q && p < order@.len() && 0 <= v < parsed.len() && dep_edge(
                            parsed@,
                            order@[p] as int,
                            v,
                        ) ==> in_order_int(order@, v),
                    forall|v: int|
                        0 <= v < cand as int && dep_edge(parsed@, cur as int, v) ==> in_order_int(
                            order@,
                            v,
                        ),
                decreases parsed.len() - cand,
            {
                let ghost old_order = order@;
                let dep = depends_on(&parsed, cur, cand);
                let seen = contains_index(&order, cand);
                proof {
                    assert(dep == dep_edge(parsed@, cur as int, cand as int));
                    assert(seen == in_order_int(old_order, cand as int));
                }
                if dep && !seen {
                    order.push(cand);
                    proof {
                        assert(dep_edge(parsed@, cur as int, cand as int));
                        assert(order@ == old_order.push(cand));
                        assert((cand as nat) < parsed@.len());
                        lemma_valid_indices_push(old_order, parsed@.len(), cand);
                        assert(valid_object_indices(order@, parsed@.len()));
                        assert(!in_order_int(old_order, cand as int));
                        lemma_unique_indices_push(old_order, cand);
                        assert(unique_indices(order@));
                        assert(cur == old_order[q as int]);
                        assert(dep_edge(parsed@, old_order[q as int] as int, cand as int));
                        lemma_non_root_parent_push(parsed@, old_order, cand, q as int);
                        assert(non_root_has_parent_edge(parsed@, order@));
                        assert(cur == order@[q as int]);
                        assert forall|p: int, v: int|
                            0 <= p < q && p < order@.len() && 0 <= v < parsed.len()
                                && dep_edge(parsed@, order@[p] as int, v) implies in_order_int(
                                order@,
                                v,
                            ) by {
                            assert(p < old_order.len());
                            assert(in_order_int(old_order, v));
                            lemma_in_order_push_preserve(old_order, v, cand);
                        };
                    }
                } else {
                    proof {
                        assert(order@ == old_order);
                        assert(non_root_has_parent_edge(parsed@, order@));
                    }
                }
                proof {
                    assert forall|v: int|
                        0 <= v < cand as int + 1 && dep_edge(parsed@, cur as int, v) implies in_order_int(
                            order@,
                            v,
                        ) by {
                        if v < cand as int {
                            assert(in_order_int(old_order, v));
                            if dep && !seen {
                                lemma_in_order_push_preserve(old_order, v, cand);
                            }
                        } else {
                            assert(v == cand as int);
                            if dep {
                                if seen {
                                    assert(in_order_int(old_order, v));
                                    assert(in_order_int(order@, v));
                                } else {
                                    lemma_in_order_push_new(old_order, cand);
                                }
                            }
                        }
                    };
                }
                cand = cand + 1;
            }
            proof {
                assert(cand == parsed.len());
                assert(non_root_has_parent_edge(parsed@, order@));
                assert forall|v: int|
                    0 <= v < parsed.len() && dep_edge(parsed@, cur as int, v) implies in_order_int(
                        order@,
                        v,
                    ) by {
                    assert(v < cand as int);
                };
                assert forall|p: int, v: int|
                    0 <= p < q + 1 && p < order@.len() && 0 <= v < parsed.len() && dep_edge(
                        parsed@,
                        order@[p] as int,
                        v,
                    ) implies in_order_int(order@, v) by {
                    if p < q {
                    } else {
                        assert(p == q);
                        assert(order@[p] as int == cur as int);
                    }
                };
            }
        }
        q = q + 1;
    }

    let mut oi: usize = 0;
    while oi < order.len()
        invariant
            oi <= order.len(),
        decreases order.len() - oi,
    {
        let obj_idx = order[oi];
        if obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        let needed_offsets = parsed[obj_idx].needed_offsets.clone();
        let need_len = needed_offsets.len();
        let mut ni: usize = 0;
        while ni < need_len
            invariant
                ni <= need_len,
                need_len == needed_offsets.len(),
            decreases need_len - ni,
        {
            assert(ni < needed_offsets.len());
            let need_off = needed_offsets[ni];
            if !has_needed_match(parsed, obj_idx, need_off) {
                return Err(LoaderError {});
            }
            ni = ni + 1;
        }
        oi = oi + 1;
    }

    proof {
        assert(valid_object_indices(order@, parsed@.len()));
        assert(unique_indices(order@));
        assert(cycle_handling_policy(order@));
        assert(parsed@.len() == 0 ==> order@.len() == 0);
        assert(parsed@.len() > 0 ==> order@.len() > 0 && order@[0] == 0);
        assert(forall|p: int, v: int|
            0 <= p < parsed.len() && p < order@.len() && 0 <= v < parsed.len() && dep_edge(
                parsed@,
                order@[p] as int,
                v,
            ) ==> in_order_int(order@, v));
        assert(direct_dep_closure(parsed@, order@));
        assert(non_root_has_parent_edge(parsed@, order@));
    }

    Ok(DiscoveryResult { order: order })
}

} // verus!
