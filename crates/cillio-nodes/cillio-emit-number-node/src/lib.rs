#[allow(warnings)]
mod bindings;

use bindings::{Error, Guest, Outputs};

struct Component;

impl Guest for Component {
    fn process() -> Result<Outputs, Error> {
        let out = Outputs { number: 42.0 };
        Ok(out)
    }
}
bindings::export!(Component with_types_in bindings);
