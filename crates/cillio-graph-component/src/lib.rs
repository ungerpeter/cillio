#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn compute() {
        println!("Graph Component says hello! To be implemented...");
    }
}

bindings::export!(Component with_types_in bindings);
