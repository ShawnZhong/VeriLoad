use vstd::prelude::*;

verus! {

#[derive(Debug)]
pub struct BinaryObject {
    pub path: Vec<u8>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct PlannerInput {
    pub objects: Vec<BinaryObject>,
}

pub open spec fn spec(input: PlannerInput, out: crate::s8_finalize_spec::RuntimePlan) -> bool {
    exists|parsed_objects: crate::s1_parse_spec::RawObjects,
           normalized_objects: crate::s2_normalize_spec::NormalizedObjects,
           dependency_graph: crate::s3_graph_spec::DependencyGraph,
           dependency_order: crate::s4_order_spec::DependencyOrder,
           symbol_bindings: crate::s5_symbol_spec::SymbolBindings,
           memory_map_plan: crate::s6_mmap_spec::MemoryMapPlan,
           relocation_writes: crate::s7_reloc_spec::RelocationWrites|
        #![auto]
    {
        &&& crate::s1_parse_spec::spec(input, Ok(parsed_objects))
        &&& crate::s2_normalize_spec::spec(
            &parsed_objects,
            Ok(normalized_objects),
        )
        &&& crate::s3_graph_spec::spec(
            &normalized_objects,
            Ok(dependency_graph),
        )
        &&& crate::s4_order_spec::spec(
            &normalized_objects,
            &dependency_graph,
            Ok(dependency_order),
        )
        &&& crate::s5_symbol_spec::spec(
            &normalized_objects,
            &dependency_order,
            Ok(symbol_bindings),
        )
        &&& crate::s6_mmap_spec::spec(
            &normalized_objects,
            &dependency_order,
            Ok(memory_map_plan),
        )
        &&& crate::s7_reloc_spec::spec(
            &normalized_objects,
            &dependency_order,
            &symbol_bindings,
            &memory_map_plan,
            Ok(relocation_writes),
        )
        &&& crate::s8_finalize_spec::spec(
            &normalized_objects,
            &dependency_order,
            &memory_map_plan,
            &relocation_writes,
            Ok(out),
        )
    }
}

}
