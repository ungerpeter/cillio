#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {
    fn addition(a: f32, b: f32) -> f32 {
        a + b
    }
}

bindings::export!(Component with_types_in bindings);
