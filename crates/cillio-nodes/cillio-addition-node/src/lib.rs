#[allow(warnings)]
mod bindings;

use bindings::{Error, Guest, Inputs, Outputs};

struct Component;

impl Guest for Component {
    fn process(in_: Inputs) -> Result<Outputs, Error> {
        let out = Outputs { sum: in_.a + in_.b };
        Ok(out)
    }
}

bindings::export!(Component with_types_in bindings);
