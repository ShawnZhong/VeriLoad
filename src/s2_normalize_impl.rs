use vstd::prelude::*;

use crate::s1_parse_spec::RawObjects;
use crate::s2_normalize_spec::{spec, NormalizedObjects};

verus! {

pub fn run(parsed: &RawObjects) -> (out: Result<NormalizedObjects, String>)
    ensures
        spec(parsed, out),
{
    // TODO(stage2): validate and normalize decoded ELF data.
    Err("TODO: stage2 normalize".to_string())
}

}
