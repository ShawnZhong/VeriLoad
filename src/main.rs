use verus_builtin::*;
use verus_builtin_macros::*;

verus! {

#[verifier::external_body]
fn print_hello() {
    println!("Hello, world!");
}

fn main() {
    print_hello();
}

} // verus!
