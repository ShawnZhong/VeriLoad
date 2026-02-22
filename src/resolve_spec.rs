use crate::discover_spec::cstr_eq_from;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn provider_pair(r: ResolvedReloc) -> Option<(usize, usize)> {
    match (r.provider_object, r.provider_symbol) {
        (Some(o), Some(s)) => Some((o, s)),
        _ => None,
    }
}

pub open spec fn symbol_match(
    parsed: Seq<ParsedObject>,
    req_obj: int,
    req_sym: int,
    prov_obj: int,
    prov_sym: int,
) -> bool {
    &&& 0 <= req_obj < parsed.len()
    &&& 0 <= req_sym < parsed[req_obj].dynsyms@.len()
    &&& 0 <= prov_obj < parsed.len()
    &&& 0 <= prov_sym < parsed[prov_obj].dynsyms@.len()
    &&& parsed[prov_obj].dynsyms@[prov_sym].st_shndx != 0
    &&& cstr_eq_from(
        parsed[req_obj].dynstr@,
        parsed[req_obj].dynsyms@[req_sym].name_offset as nat,
        parsed[prov_obj].dynstr@,
        parsed[prov_obj].dynsyms@[prov_sym].name_offset as nat,
    )
}

pub open spec fn obj_has_match(
    parsed: Seq<ParsedObject>,
    req_obj: int,
    req_sym: int,
    prov_obj: int,
) -> bool {
    exists|s: int| symbol_match(parsed, req_obj, req_sym, prov_obj, s)
}

pub open spec fn provider_result_spec(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    req_obj: int,
    req_sym: int,
    out: Option<(usize, usize)>,
) -> bool {
    match out {
        Some((prov_obj, prov_sym)) => symbol_match(
            parsed,
            req_obj,
            req_sym,
            prov_obj as int,
            prov_sym as int,
        ),
        None => forall|p: int| 0 <= p < order.len() ==> !obj_has_match(parsed, req_obj, req_sym, order[p]
            as int),
    }
}

pub open spec fn resolved_reloc_spec(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    rr: ResolvedReloc,
) -> bool {
    let req_obj = rr.requester as int;
    let req_sym = rr.sym_index as int;
    &&& rr.sym_index > 0
    &&& match (rr.provider_object, rr.provider_symbol) {
        (Some(_), None) => false,
        (None, Some(_)) => false,
        _ => true,
    }
    &&& if 0 <= req_obj < parsed.len() && 0 <= req_sym < parsed[req_obj].dynsyms@.len() {
        provider_result_spec(parsed, order, req_obj, req_sym, provider_pair(rr))
    } else {
        rr.provider_object.is_none() && rr.provider_symbol.is_none()
    }
}

pub open spec fn planned_scope_spec(load_order: Seq<usize>, planned: Seq<PlannedObject>) -> bool {
    &&& planned.len() == load_order.len()
    &&& forall|i: int|
        0 <= i < planned.len() ==> planned[i].index == load_order[i] && planned[i].base == 0
}

pub open spec fn resolve_stage_spec(
    parsed: Seq<ParsedObject>,
    discovered: DiscoveryResult,
    out: ResolutionResult,
) -> bool {
    &&& planned_scope_spec(discovered.order@, out.planned@)
    &&& forall|i: int|
        0 <= i < out.resolved_relocs@.len() ==> resolved_reloc_spec(
            parsed,
            discovered.order@,
            out.resolved_relocs@[i],
        )
}

} // verus!
