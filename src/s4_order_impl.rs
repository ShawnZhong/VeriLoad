use vstd::prelude::*;

use crate::s4_order_spec::{
    spec,
    DependencyOrder,
};
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s3_graph_spec::DependencyGraph;

verus! {

pub fn run(normalized: &NormalizedObjects, graph: &DependencyGraph) -> (out: Result<DependencyOrder, String>)
    ensures
        spec(normalized, graph, out),
{
    // TODO(stage4): compute dependency load/resolution order.
    Err("TODO: stage4 order".to_string())
}

}
