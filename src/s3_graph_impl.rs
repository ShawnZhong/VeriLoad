use vstd::prelude::*;

use crate::s3_graph_spec::{
    spec,
    DependencyGraph,
};
use crate::s2_normalize_spec::NormalizedObjects;

verus! {

pub fn run(normalized: &NormalizedObjects) -> (out: Result<DependencyGraph, String>)
    ensures
        spec(normalized, out),
{
    // TODO(stage3): build a cycle-safe dependency graph rooted at object 0.
    Err("TODO: stage3 graph".to_string())
}

}
