#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {

    fn run() -> f32 {
        42.0
    }
}

bindings::export!(Component with_types_in bindings);
