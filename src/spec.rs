use vstd::map::*;
use vstd::prelude::*;
use vstd::seq::*;

use crate::model::*;

verus! {

pub open spec fn seq_contains_nat(s: Seq<nat>, x: nat) -> bool {
    exists|i: int| 0 <= i < s.len() && s[i] == x
}

pub open spec fn seq_no_duplicates_nat(s: Seq<nat>) -> bool {
    forall|i: int, j: int| 0 <= i < s.len() && 0 <= j < s.len() && i != j ==> s[i] != s[j]
}

pub open spec fn first_index_nat(s: Seq<nat>, x: nat) -> int
    recommends
        seq_contains_nat(s, x),
{
    choose|i: int|
        0 <= i < s.len() && s[i] == x && forall|j: int| 0 <= j < i ==> s[j] != x
}

pub open spec fn seq_contains_byte(s: Seq<Byte>, b: Byte) -> bool {
    exists|i: int| 0 <= i < s.len() && s[i] == b
}

pub open spec fn is_pt_load(t: ProgramType) -> bool {
    match t {
        ProgramType::Load => true,
        _ => false,
    }
}

pub open spec fn is_pt_dynamic(t: ProgramType) -> bool {
    match t {
        ProgramType::Dynamic => true,
        _ => false,
    }
}

pub open spec fn has_program_type(obj: ParsedObject, pt: ProgramType) -> bool {
    exists|i: int| 0 <= i < obj.program_headers.len() && obj.program_headers[i].p_type == pt
}

pub open spec fn has_dyn_tag(entries: Seq<DynamicEntry>, tag: DynTag) -> bool {
    exists|i: int| 0 <= i < entries.len() && entries[i].tag == tag
}

pub open spec fn is_sym_bind_local(b: SymBind) -> bool {
    match b {
        SymBind::Local => true,
        _ => false,
    }
}

pub open spec fn is_sym_bind_global(b: SymBind) -> bool {
    match b {
        SymBind::Global => true,
        _ => false,
    }
}

pub open spec fn is_sym_bind_weak(b: SymBind) -> bool {
    match b {
        SymBind::Weak => true,
        _ => false,
    }
}

pub open spec fn load_segments_sorted(obj: ParsedObject) -> bool {
    forall|i: int, j: int|
        0 <= i < j < obj.program_headers.len()
            && is_pt_load(obj.program_headers[i].p_type)
            && is_pt_load(obj.program_headers[j].p_type)
            ==> obj.program_headers[i].vaddr <= obj.program_headers[j].vaddr
}

pub open spec fn has_load_segment(obj: ParsedObject) -> bool {
    exists|i: int| 0 <= i < obj.program_headers.len() && is_pt_load(obj.program_headers[i].p_type)
}

pub open spec fn load_segments_well_formed(page_size: nat, obj: ParsedObject) -> bool
    recommends
        page_size > 0,
{
    forall|i: int|
        0 <= i < obj.program_headers.len()
            && is_pt_load((#[trigger] obj.program_headers[i]).p_type)
            ==> {
                let ph = obj.program_headers[i];
                &&& ph.filesz <= ph.memsz
                &&& ph.offset + ph.filesz <= obj.raw.len()
                &&& (ph.align == 0 || ph.align == 1 || is_power_of_two(ph.align))
                &&& ph.vaddr % page_size == ph.offset % page_size
                &&& ph.memsz > 0
            }
}

pub open spec fn dynamic_table_well_formed(obj: ParsedObject) -> bool {
    let has_dynamic = has_program_type(obj, ProgramType::Dynamic) || obj.dynamic.len() > 0;
    &&& (has_dynamic ==> has_dyn_tag(obj.dynamic, DynTag::Null))
    &&& (has_dynamic ==> has_dyn_tag(obj.dynamic, DynTag::StrTab))
    &&& (has_dynamic ==> has_dyn_tag(obj.dynamic, DynTag::SymTab))
    &&& (has_dynamic ==> has_dyn_tag(obj.dynamic, DynTag::StrSz))
    &&& (has_dynamic ==> has_dyn_tag(obj.dynamic, DynTag::SymEnt))
    &&& (has_dyn_tag(obj.dynamic, DynTag::Rel)
        ==> (has_dyn_tag(obj.dynamic, DynTag::RelSz) && has_dyn_tag(obj.dynamic, DynTag::RelEnt)))
    &&& (has_dyn_tag(obj.dynamic, DynTag::Rela)
        ==> (has_dyn_tag(obj.dynamic, DynTag::RelaSz) && has_dyn_tag(obj.dynamic, DynTag::RelaEnt)))
    &&& (has_dyn_tag(obj.dynamic, DynTag::JmpRel)
        ==> (has_dyn_tag(obj.dynamic, DynTag::PltRel) && has_dyn_tag(obj.dynamic, DynTag::PltRelSz)))
    &&& forall|i: int| 0 <= i < obj.needed.len() ==> obj.needed[i].len() > 0
    &&& ((obj.init_fn matches Some(_)) ==> has_dyn_tag(obj.dynamic, DynTag::Init))
    &&& ((obj.fini_fn matches Some(_)) ==> has_dyn_tag(obj.dynamic, DynTag::Fini))
    &&& (obj.has_symbolic ==> has_dyn_tag(obj.dynamic, DynTag::Symbolic))
    &&& (obj.has_textrel ==> has_dyn_tag(obj.dynamic, DynTag::TextRel))
    &&& (obj.has_bind_now
        ==> (has_dyn_tag(obj.dynamic, DynTag::BindNow)
            || has_dyn_tag(obj.dynamic, DynTag::Flags)
            || has_dyn_tag(obj.dynamic, DynTag::Flags1)))
}

pub open spec fn dynsym_well_formed(obj: ParsedObject) -> bool {
    &&& obj.dynsym.len() > 0
    &&& !obj.dynsym[0].defined
    &&& forall|i: int, j: int|
        0 <= i < j < obj.dynsym.len()
            && is_sym_bind_local(obj.dynsym[j].bind)
            ==> is_sym_bind_local(obj.dynsym[i].bind)
}

pub open spec fn is_relative_reloc_kind(machine: Machine, kind: nat) -> bool {
    match machine {
        Machine::I386 => kind == R_386_RELATIVE,
        Machine::X86_64 => kind == R_X86_64_RELATIVE,
        Machine::Other(_) => false,
    }
}

pub open spec fn relocation_table_well_formed(
    obj: ParsedObject,
    rels: Seq<Relocation>,
    table_kind: RelocTableKind,
) -> bool {
    forall|i: int| 0 <= i < rels.len() ==> {
        let r = #[trigger] rels[i];
        &&& r.table == table_kind
        &&& r.sym_index < obj.dynsym.len()
        &&& (is_relative_reloc_kind(obj.machine, r.kind) ==> r.sym_index == 0)
    }
}

pub open spec fn parsed_object_ok(
    page_size: nat,
    id: ObjectId,
    src: InputObject,
    obj: ParsedObject,
) -> bool
    recommends
        page_size > 0,
{
    &&& obj.object_id == id
    &&& obj.name == src.name
    &&& obj.raw == src.bytes
    &&& has_load_segment(obj)
    &&& load_segments_sorted(obj)
    &&& load_segments_well_formed(page_size, obj)
    &&& dynamic_table_well_formed(obj)
    &&& dynsym_well_formed(obj)
    &&& relocation_table_well_formed(obj, obj.relocs, RelocTableKind::Main)
    &&& relocation_table_well_formed(obj, obj.plt_relocs, RelocTableKind::Plt)
}

pub open spec fn parse_stage_ok(input: LoaderInput, parse: ParseStageState) -> bool {
    &&& input.wf()
    &&& parse.root_id == 0
    &&& parse.objects.len() == input.objects.len()
    &&& forall|i: int| 0 <= i < parse.objects.len() ==> parsed_object_ok(
        input.page_size,
        i as nat,
        input.objects[i],
        parse.objects[i],
    )
}

pub open spec fn needed_name_matches(candidate: ParsedObject, needed: Seq<Byte>) -> bool {
    if seq_contains_byte(needed, 47u8) {
        candidate.name == needed
    } else {
        candidate.name == needed || match candidate.soname {
            Some(soname) => soname == needed,
            None => false,
        }
    }
}

pub open spec fn edge_list_contains(dep: DependencyStageState, parent: nat, child: nat) -> bool {
    dep.edges.dom().contains(parent) && seq_contains_nat(dep.edges.index(parent), child)
}

pub open spec fn dependency_edges_well_formed(parse: ParseStageState, dep: DependencyStageState) -> bool {
    forall|from: nat| from < parse.objects.len() && dep.edges.dom().contains(from) ==> {
        &&& dep.edges.index(from).len() == parse.objects[from as int].needed.len()
        &&& forall|k: int| 0 <= k < dep.edges.index(from).len() ==> {
            let child = #[trigger] dep.edges.index(from)[k];
            child < parse.objects.len() && needed_name_matches(
                parse.objects[child as int],
                parse.objects[from as int].needed[k],
            )
        }
    }
}

pub open spec fn dependency_order_well_formed(parse: ParseStageState, dep: DependencyStageState) -> bool {
    &&& dep.bfs_order.len() > 0
    &&& dep.bfs_order[0] == dep.root_id
    &&& dep.root_id < parse.objects.len()
    &&& seq_no_duplicates_nat(dep.bfs_order)
    &&& forall|i: int| 0 <= i < dep.bfs_order.len() ==> dep.bfs_order[i] < parse.objects.len()
    &&& forall|i: int| 1 <= i < dep.bfs_order.len() ==> exists|j: int|
        0 <= j < i
            && edge_list_contains(dep, dep.bfs_order[j], #[trigger] dep.bfs_order[i])
}

pub open spec fn all_edges_appear_in_bfs(parse: ParseStageState, dep: DependencyStageState) -> bool {
    forall|from: nat| from < parse.objects.len() && dep.edges.dom().contains(from) ==> {
        forall|k: int| 0 <= k < dep.edges.index(from).len() ==> seq_contains_nat(
            dep.bfs_order,
            dep.edges.index(from)[k],
        )
    }
}

pub open spec fn dependency_stage_ok(parse: ParseStageState, dep: DependencyStageState) -> bool {
    &&& dep.root_id == parse.root_id
    &&& dependency_edges_well_formed(parse, dep)
    &&& dependency_order_well_formed(parse, dep)
    &&& all_edges_appear_in_bfs(parse, dep)
}

pub open spec fn ranges_disjoint(a_start: nat, a_len: nat, b_start: nat, b_len: nat) -> bool {
    a_start + a_len <= b_start || b_start + b_len <= a_start
}

pub open spec fn segment_plan_matches_header(
    parse: ParseStageState,
    layout: LayoutStageState,
    plan: SegmentMapPlan,
) -> bool {
    &&& plan.object_id < parse.objects.len()
    &&& layout.load_bias.dom().contains(plan.object_id)
    &&& plan.ph_index < parse.objects[plan.object_id as int].program_headers.len()
    &&& {
        let obj = parse.objects[plan.object_id as int];
        let ph = obj.program_headers[plan.ph_index as int];
        &&& is_pt_load(ph.p_type)
        &&& plan.start == layout.load_bias.index(plan.object_id) + ph.vaddr
        &&& plan.prot == ph.flags
        &&& ph.filesz <= ph.memsz
        &&& ph.offset + ph.filesz <= obj.raw.len()
        &&& plan.bytes.len() == ph.memsz
        &&& plan.bytes.subrange(0, ph.filesz as int) == obj.raw.subrange(
            ph.offset as int,
            (ph.offset + ph.filesz) as int,
        )
        &&& forall|k: int| ph.filesz <= k < ph.memsz ==> plan.bytes[k] == 0u8
    }
}

pub open spec fn layout_load_bias_well_formed(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    page_size: nat,
) -> bool
    recommends
        page_size > 0,
{
    &&& forall|i: int| 0 <= i < dep.bfs_order.len() ==> layout.load_bias.dom().contains(dep.bfs_order[i])
    &&& forall|obj: nat| obj < parse.objects.len() && layout.load_bias.dom().contains(obj)
        ==> seq_contains_nat(dep.bfs_order, obj)
    &&& forall|obj: nat|
        obj < parse.objects.len() && seq_contains_nat(dep.bfs_order, obj) ==> {
            let o = parse.objects[obj as int];
            if o.elf_type == ElfType::Exec {
                layout.load_bias.index(obj) == 0
            } else {
                align_down(layout.load_bias.index(obj), page_size) == layout.load_bias.index(obj)
            }
        }
}

pub open spec fn layout_has_every_load_segment(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
) -> bool {
    forall|obj: nat, ph_i: int|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && 0 <= ph_i < parse.objects[obj as int].program_headers.len()
            && is_pt_load(parse.objects[obj as int].program_headers[ph_i].p_type)
            ==> exists|pi: int|
                0 <= pi < layout.segment_plans.len()
                    && layout.segment_plans[pi].object_id == obj
                    && layout.segment_plans[pi].ph_index == ph_i as nat
                    && forall|pj: int|
                        0 <= pj < layout.segment_plans.len()
                            && layout.segment_plans[pj].object_id == obj
                            && layout.segment_plans[pj].ph_index == ph_i as nat
                            ==> pj == pi
}

pub open spec fn layout_plans_non_overlapping(layout: LayoutStageState) -> bool {
    forall|i: int, j: int|
        0 <= i < j < layout.segment_plans.len()
            ==> ranges_disjoint(
                layout.segment_plans[i].start,
                layout.segment_plans[i].bytes.len(),
                layout.segment_plans[j].start,
                layout.segment_plans[j].bytes.len(),
            )
}

pub open spec fn layout_stage_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    page_size: nat,
) -> bool
    recommends
        page_size > 0,
{
    &&& layout.root_id == dep.root_id
    &&& layout_load_bias_well_formed(parse, dep, layout, page_size)
    &&& forall|i: int| 0 <= i < layout.segment_plans.len()
        ==> seq_contains_nat(dep.bfs_order, layout.segment_plans[i].object_id)
            && segment_plan_matches_header(parse, layout, layout.segment_plans[i])
    &&& layout_has_every_load_segment(parse, dep, layout)
    &&& layout_plans_non_overlapping(layout)
}

pub open spec fn reloc_ref_valid(parse: ParseStageState, dep: DependencyStageState, r: RelocRef) -> bool {
    &&& r.object_id < parse.objects.len()
    &&& seq_contains_nat(dep.bfs_order, r.object_id)
    &&& match r.table {
        RelocTableKind::Main => r.rel_index < parse.objects[r.object_id as int].relocs.len(),
        RelocTableKind::Plt => r.rel_index < parse.objects[r.object_id as int].plt_relocs.len(),
    }
}

pub open spec fn reloc_ref_index_in_bounds(parse: ParseStageState, r: RelocRef) -> bool {
    &&& r.object_id < parse.objects.len()
    &&& match r.table {
        RelocTableKind::Main => r.rel_index < parse.objects[r.object_id as int].relocs.len(),
        RelocTableKind::Plt => r.rel_index < parse.objects[r.object_id as int].plt_relocs.len(),
    }
}

pub open spec fn relocation_at(parse: ParseStageState, r: RelocRef) -> Relocation
    recommends
        reloc_ref_index_in_bounds(parse, r),
{
    match r.table {
        RelocTableKind::Main => parse.objects[r.object_id as int].relocs[r.rel_index as int],
        RelocTableKind::Plt => parse.objects[r.object_id as int].plt_relocs[r.rel_index as int],
    }
}

pub open spec fn symbol_is_runtime_definition(root_id: nat, obj_id: nat, sym: DynSymbol) -> bool {
    if sym.defined {
        is_sym_bind_global(sym.bind) || is_sym_bind_weak(sym.bind)
    } else {
        // System V special case: executable undefined STT_FUNC with non-zero st_value
        obj_id == root_id
            && sym.sym_type == SymType::Func
            && sym.value != 0
            && (is_sym_bind_global(sym.bind) || is_sym_bind_weak(sym.bind))
    }
}

pub open spec fn object_has_name_with_bind(
    parse: ParseStageState,
    root_id: nat,
    obj_id: nat,
    name: Seq<Byte>,
    want_global: bool,
) -> bool
    recommends
        obj_id < parse.objects.len(),
{
    exists|si: int|
        0 <= si < parse.objects[obj_id as int].dynsym.len()
            && parse.objects[obj_id as int].dynsym[si].name == name
            && if want_global {
                is_sym_bind_global(parse.objects[obj_id as int].dynsym[si].bind)
            } else {
                is_sym_bind_weak(parse.objects[obj_id as int].dynsym[si].bind)
            }
            && symbol_is_runtime_definition(root_id, obj_id, parse.objects[obj_id as int].dynsym[si])
}

pub open spec fn object_has_any_definition_for_name(
    parse: ParseStageState,
    root_id: nat,
    obj_id: nat,
    name: Seq<Byte>,
) -> bool
    recommends
        obj_id < parse.objects.len(),
{
    object_has_name_with_bind(parse, root_id, obj_id, name, true)
        || object_has_name_with_bind(parse, root_id, obj_id, name, false)
}

pub open spec fn requester_symbolic_override(
    parse: ParseStageState,
    dep: DependencyStageState,
    r: RelocRef,
    name: Seq<Byte>,
) -> bool
    recommends
        r.object_id < parse.objects.len(),
{
    let req = parse.objects[r.object_id as int];
    r.object_id != dep.root_id
        && req.elf_type == ElfType::Dyn
        && req.has_symbolic
        && object_has_any_definition_for_name(parse, dep.root_id, r.object_id, name)
}

pub open spec fn no_global_definition_anywhere(
    parse: ParseStageState,
    dep: DependencyStageState,
    name: Seq<Byte>,
) -> bool {
    forall|i: int| 0 <= i < dep.bfs_order.len()
        ==> !object_has_name_with_bind(parse, dep.root_id, dep.bfs_order[i], name, true)
}

pub open spec fn no_weak_definition_anywhere(
    parse: ParseStageState,
    dep: DependencyStageState,
    name: Seq<Byte>,
) -> bool {
    forall|i: int| 0 <= i < dep.bfs_order.len()
        ==> !object_has_name_with_bind(parse, dep.root_id, dep.bfs_order[i], name, false)
}

pub open spec fn no_earlier_global_definition(
    parse: ParseStageState,
    dep: DependencyStageState,
    provider: nat,
    name: Seq<Byte>,
) -> bool
    recommends
        seq_contains_nat(dep.bfs_order, provider),
{
    let p = first_index_nat(dep.bfs_order, provider);
    forall|i: int| 0 <= i < p
        ==> !object_has_name_with_bind(parse, dep.root_id, dep.bfs_order[i], name, true)
}

pub open spec fn no_earlier_weak_definition(
    parse: ParseStageState,
    dep: DependencyStageState,
    provider: nat,
    name: Seq<Byte>,
) -> bool
    recommends
        seq_contains_nat(dep.bfs_order, provider),
{
    let p = first_index_nat(dep.bfs_order, provider);
    forall|i: int| 0 <= i < p
        ==> !object_has_name_with_bind(parse, dep.root_id, dep.bfs_order[i], name, false)
}

pub open spec fn runtime_symbol_addr(
    layout: LayoutStageState,
    obj_id: nat,
    sym: DynSymbol,
) -> Addr
    recommends
        layout.load_bias.dom().contains(obj_id),
{
    layout.load_bias.index(obj_id) + sym.value
}

pub open spec fn symbol_resolution_entry_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
    r: RelocRef,
) -> bool
    recommends
        reloc_ref_valid(parse, dep, r),
        sym_stage.resolutions.dom().contains(r),
{
    let rel = relocation_at(parse, r);
    if rel.sym_index >= parse.objects[r.object_id as int].dynsym.len() {
        false
    } else {
        let req_sym = parse.objects[r.object_id as int].dynsym[rel.sym_index as int];
        match sym_stage.resolutions.index(r) {
            SymbolResolution::Resolved(target) => {
                &&& target.object_id < parse.objects.len()
                &&& seq_contains_nat(dep.bfs_order, target.object_id)
                &&& layout.load_bias.dom().contains(target.object_id)
                &&& target.sym_index < parse.objects[target.object_id as int].dynsym.len()
                &&& {
                    let psym = parse.objects[target.object_id as int].dynsym[target.sym_index as int];
                    &&& psym.name == req_sym.name
                    &&& symbol_is_runtime_definition(dep.root_id, target.object_id, psym)
                    &&& target.bind == psym.bind
                    &&& target.addr == runtime_symbol_addr(layout, target.object_id, psym)
                    &&& if requester_symbolic_override(parse, dep, r, req_sym.name) {
                        target.object_id == r.object_id
                    } else if is_sym_bind_global(psym.bind) {
                        no_earlier_global_definition(parse, dep, target.object_id, req_sym.name)
                    } else {
                        &&& is_sym_bind_weak(psym.bind)
                        &&& no_global_definition_anywhere(parse, dep, req_sym.name)
                        &&& no_earlier_weak_definition(parse, dep, target.object_id, req_sym.name)
                    }
                }
            },
            SymbolResolution::UnresolvedWeakZero => {
                &&& is_sym_bind_weak(req_sym.bind)
                &&& !requester_symbolic_override(parse, dep, r, req_sym.name)
                &&& no_global_definition_anywhere(parse, dep, req_sym.name)
                &&& no_weak_definition_anywhere(parse, dep, req_sym.name)
            },
        }
    }
}

pub open spec fn symbol_stage_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
) -> bool {
    &&& forall|obj: nat, ri: nat|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && ri < parse.objects[obj as int].relocs.len()
            && parse.objects[obj as int].relocs[ri as int].sym_index > 0
            ==> {
                let r = RelocRef { object_id: obj, table: RelocTableKind::Main, rel_index: ri };
                sym_stage.resolutions.dom().contains(r)
                    && symbol_resolution_entry_ok(parse, dep, layout, sym_stage, r)
            }
    &&& forall|obj: nat, ri: nat|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && ri < parse.objects[obj as int].plt_relocs.len()
            && parse.objects[obj as int].plt_relocs[ri as int].sym_index > 0
            ==> {
                let r = RelocRef { object_id: obj, table: RelocTableKind::Plt, rel_index: ri };
                sym_stage.resolutions.dom().contains(r)
                    && symbol_resolution_entry_ok(parse, dep, layout, sym_stage, r)
            }
}

pub open spec fn runtime_relocation_width(machine: Machine, kind: nat) -> Option<nat> {
    match machine {
        Machine::I386 => {
            if kind == R_386_GLOB_DAT || kind == R_386_JMP_SLOT || kind == R_386_RELATIVE {
                Some(4)
            } else {
                None
            }
        },
        Machine::X86_64 => {
            if kind == R_X86_64_64 {
                Some(8)
            } else if kind == R_X86_64_PC32 || kind == R_X86_64_PLT32 {
                Some(4)
            } else if kind == R_X86_64_GLOB_DAT || kind == R_X86_64_JUMP_SLOT || kind == R_X86_64_RELATIVE {
                Some(8)
            } else {
                None
            }
        },
        Machine::Other(_) => None,
    }
}

pub open spec fn runtime_relocation_value(
    machine: Machine,
    kind: nat,
    s: int,
    a: int,
    p: int,
    b: int,
) -> Option<int> {
    match machine {
        Machine::I386 => {
            if kind == R_386_GLOB_DAT || kind == R_386_JMP_SLOT {
                Some(s)
            } else if kind == R_386_RELATIVE {
                Some(b + a)
            } else {
                None
            }
        },
        Machine::X86_64 => {
            if kind == R_X86_64_64 {
                Some(s + a)
            } else if kind == R_X86_64_PC32 || kind == R_X86_64_PLT32 {
                Some(s + a - p)
            } else if kind == R_X86_64_GLOB_DAT || kind == R_X86_64_JUMP_SLOT {
                Some(s)
            } else if kind == R_X86_64_RELATIVE {
                Some(b + a)
            } else {
                None
            }
        },
        Machine::Other(_) => None,
    }
}

pub open spec fn relocation_symbol_value(
    parse: ParseStageState,
    sym_stage: SymbolResolutionStageState,
    r: RelocRef,
) -> Option<int>
    recommends
        reloc_ref_index_in_bounds(parse, r),
{
    let rel = relocation_at(parse, r);
    if rel.sym_index == 0 {
        Some(0)
    } else if sym_stage.resolutions.dom().contains(r) {
        match sym_stage.resolutions.index(r) {
            SymbolResolution::Resolved(target) => Some(target.addr as int),
            SymbolResolution::UnresolvedWeakZero => Some(0),
        }
    } else {
        None
    }
}

pub open spec fn write_matches_reloc(
    parse: ParseStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
    w: RelocationWrite,
    r: RelocRef,
) -> bool
    recommends
        reloc_ref_index_in_bounds(parse, r),
        layout.load_bias.dom().contains(r.object_id),
{
    let rel = relocation_at(parse, r);
    let machine = parse.objects[r.object_id as int].machine;
    let p_nat = layout.load_bias.index(r.object_id) + rel.offset;
    let p = p_nat as int;
    let b = layout.load_bias.index(r.object_id) as int;
    match (relocation_symbol_value(parse, sym_stage, r), runtime_relocation_width(machine, rel.kind)) {
        (Some(s), Some(width)) => match runtime_relocation_value(machine, rel.kind, s, rel.addend, p, b) {
            Some(v) => {
                &&& w.object_id == r.object_id
                &&& w.vaddr == p_nat
                &&& w.width == width
                &&& w.computed_value == v
            },
            None => false,
        },
        _ => false,
    }
}

pub open spec fn relocation_exists_unique_write(
    parse: ParseStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
    reloc_stage: RelocationStageState,
    r: RelocRef,
) -> bool
    recommends
        reloc_ref_index_in_bounds(parse, r),
        layout.load_bias.dom().contains(r.object_id),
{
    exists|wi: int|
        0 <= wi < reloc_stage.writes.len()
            && write_matches_reloc(parse, layout, sym_stage, reloc_stage.writes[wi], r)
            && forall|wj: int|
                0 <= wj < reloc_stage.writes.len()
                    && write_matches_reloc(parse, layout, sym_stage, reloc_stage.writes[wj], r)
                    ==> wj == wi
}

pub open spec fn write_fits_plan(w: RelocationWrite, plan: SegmentMapPlan) -> bool {
    plan.start <= w.vaddr && w.vaddr + w.width <= plan.start + plan.bytes.len()
}

pub open spec fn writes_are_in_bounds(reloc_stage: RelocationStageState) -> bool {
    forall|wi: int| 0 <= wi < reloc_stage.writes.len() ==> exists|pi: int|
        0 <= pi < reloc_stage.relocated_plans.len()
            && write_fits_plan(#[trigger] reloc_stage.writes[wi], reloc_stage.relocated_plans[pi])
}

pub open spec fn relocation_stage_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
    reloc_stage: RelocationStageState,
) -> bool {
    &&& reloc_stage.relocated_plans.len() == layout.segment_plans.len()
    &&& forall|i: int| 0 <= i < layout.segment_plans.len() ==> {
        let before = #[trigger] layout.segment_plans[i];
        let after = reloc_stage.relocated_plans[i];
        &&& after.object_id == before.object_id
        &&& after.ph_index == before.ph_index
        &&& after.start == before.start
        &&& after.prot == before.prot
        &&& after.bytes.len() == before.bytes.len()
    }
    &&& forall|obj: nat, ri: nat|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && layout.load_bias.dom().contains(obj)
            && ri < parse.objects[obj as int].relocs.len()
            ==> relocation_exists_unique_write(
                parse,
                layout,
                sym_stage,
                reloc_stage,
                RelocRef { object_id: obj, table: RelocTableKind::Main, rel_index: ri },
            )
    &&& forall|obj: nat, ri: nat|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && layout.load_bias.dom().contains(obj)
            && ri < parse.objects[obj as int].plt_relocs.len()
            ==> relocation_exists_unique_write(
                parse,
                layout,
                sym_stage,
                reloc_stage,
                RelocRef { object_id: obj, table: RelocTableKind::Plt, rel_index: ri },
            )
    &&& writes_are_in_bounds(reloc_stage)
}

pub open spec fn object_has_init(obj: ParsedObject) -> bool {
    obj.init_fn matches Some(_)
}

pub open spec fn call_contains_object(calls: Seq<InitializerCall>, obj_id: nat) -> bool {
    exists|i: int| 0 <= i < calls.len() && calls[i].object_id == obj_id
}

pub open spec fn first_call_index(calls: Seq<InitializerCall>, obj_id: nat) -> int
    recommends
        call_contains_object(calls, obj_id),
{
    choose|i: int|
        0 <= i < calls.len()
            && calls[i].object_id == obj_id
            && forall|j: int| 0 <= j < i ==> calls[j].object_id != obj_id
}

pub open spec fn init_stage_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    init_stage: InitStageState,
) -> bool {
    &&& forall|i: int, j: int|
        0 <= i < j < init_stage.init_calls.len()
            ==> init_stage.init_calls[i].object_id != init_stage.init_calls[j].object_id
    &&& forall|i: int| 0 <= i < init_stage.init_calls.len() ==> {
        let c = #[trigger] init_stage.init_calls[i];
        &&& c.object_id < parse.objects.len()
        &&& seq_contains_nat(dep.bfs_order, c.object_id)
        &&& c.object_id != dep.root_id
        &&& object_has_init(parse.objects[c.object_id as int])
        &&& layout.load_bias.dom().contains(c.object_id)
        &&& match parse.objects[c.object_id as int].init_fn {
            Some(init_pc) => c.pc == layout.load_bias.index(c.object_id) + init_pc,
            None => false,
        }
    }
    &&& forall|obj: nat|
        obj < parse.objects.len()
            && seq_contains_nat(dep.bfs_order, obj)
            && obj != dep.root_id
            && object_has_init(parse.objects[obj as int])
            ==> call_contains_object(init_stage.init_calls, obj)
    &&& forall|obj: nat|
        obj < parse.objects.len()
            && (!seq_contains_nat(dep.bfs_order, obj)
                || obj == dep.root_id
                || !object_has_init(parse.objects[obj as int]))
            ==> !call_contains_object(init_stage.init_calls, obj)
    &&& forall|a: nat|
        a < parse.objects.len()
            && dep.edges.dom().contains(a)
            && a != dep.root_id
            && object_has_init(parse.objects[a as int])
            && call_contains_object(init_stage.init_calls, a)
            ==> forall|k: int| 0 <= k < dep.edges.index(a).len() ==> {
                let b = #[trigger] dep.edges.index(a)[k];
                if b != dep.root_id
                    && object_has_init(parse.objects[b as int])
                    && call_contains_object(init_stage.init_calls, b) {
                    first_call_index(init_stage.init_calls, b) < first_call_index(init_stage.init_calls, a)
                } else {
                    true
                }
            }
}

pub open spec fn finalize_stage_ok(
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    reloc_stage: RelocationStageState,
    init_stage: InitStageState,
    out: LoaderOutput,
) -> bool {
    &&& layout.load_bias.dom().contains(dep.root_id)
    &&& out.entry_pc == layout.load_bias.index(dep.root_id) + parse.objects[dep.root_id as int].entry
    &&& out.initializers == init_stage.init_calls
    &&& out.mmap_plans.len() == reloc_stage.relocated_plans.len()
    &&& forall|i: int| 0 <= i < out.mmap_plans.len() ==> {
        &&& out.mmap_plans[i].start == reloc_stage.relocated_plans[i].start
        &&& out.mmap_plans[i].bytes == reloc_stage.relocated_plans[i].bytes
        &&& out.mmap_plans[i].prot == reloc_stage.relocated_plans[i].prot
    }
    &&& out.wf()
}

pub open spec fn loader_spec(input: LoaderInput, out: LoaderOutput) -> bool {
    exists|parse: ParseStageState,
        dep: DependencyStageState,
        layout: LayoutStageState,
        sym_stage: SymbolResolutionStageState,
        reloc_stage: RelocationStageState,
        init_stage: InitStageState|
        parse_stage_ok(input, parse)
            && dependency_stage_ok(parse, dep)
            && layout_stage_ok(parse, dep, layout, input.page_size)
            && symbol_stage_ok(parse, dep, layout, sym_stage)
            && relocation_stage_ok(parse, dep, layout, sym_stage, reloc_stage)
            && init_stage_ok(parse, dep, layout, init_stage)
            && finalize_stage_ok(parse, dep, layout, reloc_stage, init_stage, out)
}

} // verus!
