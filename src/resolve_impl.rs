use crate::discover_spec::cstr_eq_from;
use crate::consts::*;
use crate::resolve_spec::*;
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

fn symbol_match_exec(
    parsed: &Vec<ParsedObject>,
    req_obj: usize,
    req_sym: usize,
    prov_obj: usize,
    prov_sym: usize,
) -> (r: bool)
    requires
        req_obj < parsed@.len(),
        req_sym < parsed@[req_obj as int].dynsyms@.len(),
        prov_obj < parsed@.len(),
        prov_sym < parsed@[prov_obj as int].dynsyms@.len(),
    ensures
        r == symbol_match(
            parsed@,
            req_obj as int,
            req_sym as int,
            prov_obj as int,
            prov_sym as int,
        ),
{
    let req_name = parsed[req_obj].dynsyms[req_sym].name_offset;
    let prov_sym_rec = &parsed[prov_obj].dynsyms[prov_sym];
    let prov_name = prov_sym_rec.name_offset;
    if prov_sym_rec.st_shndx == 0 {
        false
    } else {
        cstr_eq_from_exec(
            &parsed[req_obj].dynstr,
            req_name as usize,
            &parsed[prov_obj].dynstr,
            prov_name as usize,
        )
    }
}

fn find_provider(
    parsed: &Vec<ParsedObject>,
    order: &Vec<usize>,
    req_obj: usize,
    req_sym: usize,
) -> (r: Option<(usize, usize)>)
    requires
        req_obj < parsed@.len(),
        req_sym < parsed@[req_obj as int].dynsyms@.len(),
    ensures
        provider_result_spec(parsed@, order@, req_obj as int, req_sym as int, r),
{
    proof {
        assert(req_obj < parsed.len());
    }

    let mut pos: usize = 0;
    while pos < order.len()
        invariant
            pos <= order.len(),
            req_obj < parsed.len(),
            req_sym < parsed@[req_obj as int].dynsyms@.len(),
            forall|p: int|
                0 <= p < pos ==> !obj_has_match(
                    parsed@,
                    req_obj as int,
                    req_sym as int,
                    order@[p] as int,
                ),
        decreases order.len() - pos,
    {
        let cand_obj = order[pos];
        if cand_obj < parsed.len() {
            let mut s: usize = 0;
            while s < parsed[cand_obj].dynsyms.len()
                invariant
                    s <= parsed@[cand_obj as int].dynsyms@.len(),
                    req_obj < parsed.len(),
                    req_sym < parsed@[req_obj as int].dynsyms@.len(),
                    cand_obj < parsed.len(),
                    forall|p: int|
                        0 <= p < pos ==> !obj_has_match(
                            parsed@,
                            req_obj as int,
                            req_sym as int,
                            order@[p] as int,
                        ),
                    forall|s0: int|
                        0 <= s0 < s ==> !symbol_match(
                            parsed@,
                            req_obj as int,
                            req_sym as int,
                            cand_obj as int,
                            s0,
                        ),
                decreases parsed@[cand_obj as int].dynsyms@.len() - s,
            {
                let m = symbol_match_exec(parsed, req_obj, req_sym, cand_obj, s);
                if m {
                    proof {
                        assert(symbol_match(
                            parsed@,
                            req_obj as int,
                            req_sym as int,
                            cand_obj as int,
                            s as int,
                        ));
                        assert(provider_result_spec(
                            parsed@,
                            order@,
                            req_obj as int,
                            req_sym as int,
                            Some((cand_obj, s)),
                        ));
                    }
                    return Some((cand_obj, s));
                }
                proof {
                    assert(!symbol_match(
                        parsed@,
                        req_obj as int,
                        req_sym as int,
                        cand_obj as int,
                        s as int,
                    ));
                }
                s = s + 1;
            }
            proof {
                assert(s == parsed@[cand_obj as int].dynsyms@.len());
                assert(!obj_has_match(parsed@, req_obj as int, req_sym as int, cand_obj as int)) by {
                    if obj_has_match(parsed@, req_obj as int, req_sym as int, cand_obj as int) {
                        let s0 = choose|s0: int|
                            symbol_match(
                                parsed@,
                                req_obj as int,
                                req_sym as int,
                                cand_obj as int,
                                s0,
                            );
                        assert(s0 < s as int);
                        assert(!symbol_match(
                            parsed@,
                            req_obj as int,
                            req_sym as int,
                            cand_obj as int,
                            s0,
                        ));
                        assert(false);
                    }
                };
            }
        }
        pos = pos + 1;
    }

    proof {
        assert(pos == order.len());
        assert(provider_result_spec(
            parsed@,
            order@,
            req_obj as int,
            req_sym as int,
            None,
        ));
    }
    None
}

fn symbol_is_weak_undef(sym: &DynSymbol) -> bool {
    let bind = sym.st_info >> 4;
    bind == 2 && sym.st_shndx == 0
}

fn symbol_relocation_requires_provider(rel_type: u32, sym: &DynSymbol) -> bool {
    if rel_type == R_X86_64_COPY {
        true
    } else {
        (rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT || rel_type
            == R_X86_64_64) && !symbol_is_weak_undef(sym)
    }
}

pub fn resolve_stage_ref(
    parsed: &Vec<ParsedObject>,
    discovered: &DiscoveryResult,
) -> (out: Result<ResolutionResult, LoaderError>)
    ensures
        out.is_ok() ==> resolve_stage_spec(parsed@, *discovered, out.unwrap()),
{
    let mut planned: Vec<PlannedObject> = Vec::new();
    let mut pi: usize = 0;
    while pi < discovered.order.len()
        invariant
            pi <= discovered.order.len(),
            planned@.len() == pi,
            forall|k: int|
                0 <= k < planned@.len() ==> planned@[k].index == discovered.order@[k] && planned@[k].base
                    == 0,
        decreases discovered.order.len() - pi,
    {
        let idx = discovered.order[pi];
        if idx >= parsed.len() {
            return Err(LoaderError {});
        }
        planned.push(PlannedObject { index: idx, base: 0 });
        pi = pi + 1;
    }

    let mut resolved_relocs: Vec<ResolvedReloc> = Vec::new();
    let mut oi: usize = 0;
    while oi < discovered.order.len()
        invariant
            oi <= discovered.order.len(),
            forall|k: int|
                0 <= k < resolved_relocs@.len() ==> resolved_reloc_spec(
                    parsed@,
                    discovered.order@,
                    resolved_relocs@[k],
                ),
        decreases discovered.order.len() - oi,
    {
        let obj_idx = discovered.order[oi];
        if obj_idx < parsed.len() {
            let mut ri: usize = 0;
            while ri < parsed[obj_idx].relas.len()
                invariant
                    ri <= parsed@[obj_idx as int].relas@.len(),
                    oi < discovered.order.len(),
                    obj_idx == discovered.order@[oi as int],
                    obj_idx < parsed.len(),
                    forall|k: int|
                        0 <= k < resolved_relocs@.len() ==> resolved_reloc_spec(
                            parsed@,
                            discovered.order@,
                            resolved_relocs@[k],
                        ),
                decreases parsed@[obj_idx as int].relas@.len() - ri,
            {
                let rel = &parsed[obj_idx].relas[ri];
                let rel_type = rel.reloc_type();
                let sym_idx = rel.sym_index();
                if rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT
                    || rel_type == R_X86_64_COPY || rel_type == R_X86_64_64
                {
                    if sym_idx == 0 || sym_idx >= parsed[obj_idx].dynsyms.len() {
                        return Err(LoaderError {});
                    }
                }
                if sym_idx > 0 {
                    let mut prov: Option<(usize, usize)> = None;
                    let mut provider_required = false;
                    if sym_idx < parsed[obj_idx].dynsyms.len() {
                        proof {
                            assert(sym_idx < parsed@[obj_idx as int].dynsyms@.len());
                        }
                        provider_required = symbol_relocation_requires_provider(rel_type, &parsed[obj_idx].dynsyms[sym_idx]);
                        prov = find_provider(parsed, &discovered.order, obj_idx, sym_idx);
                    }
                    if provider_required && prov.is_none() {
                        return Err(LoaderError {});
                    }

                    let new_rr = match prov {
                        Some((po, ps)) => ResolvedReloc {
                            requester: obj_idx,
                            is_jmprel: false,
                            reloc_index: ri,
                            sym_index: sym_idx,
                            provider_object: Some(po),
                            provider_symbol: Some(ps),
                        },
                        None => ResolvedReloc {
                            requester: obj_idx,
                            is_jmprel: false,
                            reloc_index: ri,
                            sym_index: sym_idx,
                            provider_object: None,
                            provider_symbol: None,
                        },
                    };

                    let ghost old_rrs = resolved_relocs@;
                    resolved_relocs.push(new_rr);
                    proof {
                        assert forall|k: int|
                            0 <= k < resolved_relocs@.len() implies resolved_reloc_spec(
                                parsed@,
                                discovered.order@,
                                resolved_relocs@[k],
                            ) by {
                            if k < old_rrs.len() {
                            } else {
                                assert(k == old_rrs.len());
                                if sym_idx < parsed@[obj_idx as int].dynsyms@.len() {
                                    assert(provider_result_spec(
                                        parsed@,
                                        discovered.order@,
                                        obj_idx as int,
                                        sym_idx as int,
                                        prov,
                                    ));
                                } else {
                                    assert(new_rr.provider_object.is_none());
                                    assert(new_rr.provider_symbol.is_none());
                                }
                                assert(resolved_reloc_spec(
                                    parsed@,
                                    discovered.order@,
                                    new_rr,
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
                    oi < discovered.order.len(),
                    obj_idx == discovered.order@[oi as int],
                    obj_idx < parsed.len(),
                    forall|k: int|
                        0 <= k < resolved_relocs@.len() ==> resolved_reloc_spec(
                            parsed@,
                            discovered.order@,
                            resolved_relocs@[k],
                        ),
                decreases parsed@[obj_idx as int].jmprels@.len() - ji,
            {
                let rel = &parsed[obj_idx].jmprels[ji];
                let rel_type = rel.reloc_type();
                let sym_idx = rel.sym_index();
                if rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT
                    || rel_type == R_X86_64_COPY || rel_type == R_X86_64_64
                {
                    if sym_idx == 0 || sym_idx >= parsed[obj_idx].dynsyms.len() {
                        return Err(LoaderError {});
                    }
                }
                if sym_idx > 0 {
                    let mut prov: Option<(usize, usize)> = None;
                    let mut provider_required = false;
                    if sym_idx < parsed[obj_idx].dynsyms.len() {
                        proof {
                            assert(sym_idx < parsed@[obj_idx as int].dynsyms@.len());
                        }
                        provider_required = symbol_relocation_requires_provider(rel_type, &parsed[obj_idx].dynsyms[sym_idx]);
                        prov = find_provider(parsed, &discovered.order, obj_idx, sym_idx);
                    }
                    if provider_required && prov.is_none() {
                        return Err(LoaderError {});
                    }

                    let new_rr = match prov {
                        Some((po, ps)) => ResolvedReloc {
                            requester: obj_idx,
                            is_jmprel: true,
                            reloc_index: ji,
                            sym_index: sym_idx,
                            provider_object: Some(po),
                            provider_symbol: Some(ps),
                        },
                        None => ResolvedReloc {
                            requester: obj_idx,
                            is_jmprel: true,
                            reloc_index: ji,
                            sym_index: sym_idx,
                            provider_object: None,
                            provider_symbol: None,
                        },
                    };

                    let ghost old_rrs = resolved_relocs@;
                    resolved_relocs.push(new_rr);
                    proof {
                        assert forall|k: int|
                            0 <= k < resolved_relocs@.len() implies resolved_reloc_spec(
                                parsed@,
                                discovered.order@,
                                resolved_relocs@[k],
                            ) by {
                            if k < old_rrs.len() {
                            } else {
                                assert(k == old_rrs.len());
                                if sym_idx < parsed@[obj_idx as int].dynsyms@.len() {
                                    assert(provider_result_spec(
                                        parsed@,
                                        discovered.order@,
                                        obj_idx as int,
                                        sym_idx as int,
                                        prov,
                                    ));
                                } else {
                                    assert(new_rr.provider_object.is_none());
                                    assert(new_rr.provider_symbol.is_none());
                                }
                                assert(resolved_reloc_spec(
                                    parsed@,
                                    discovered.order@,
                                    new_rr,
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
        oi = oi + 1;
    }

    proof {
        assert(planned_scope_spec(discovered.order@, planned@));
        assert(forall|k: int|
            0 <= k < resolved_relocs@.len() ==> resolved_reloc_spec(
                parsed@,
                discovered.order@,
                resolved_relocs@[k],
            ));
    }

    Ok(ResolutionResult { planned, resolved_relocs })
}

pub fn resolve_stage(
    parsed: Vec<ParsedObject>,
    discovered: DiscoveryResult,
) -> (out: Result<ResolutionResult, LoaderError>)
    ensures
        out.is_ok() ==> resolve_stage_spec(parsed@, discovered, out.unwrap()),
{
    resolve_stage_ref(&parsed, &discovered)
}

} // verus!
