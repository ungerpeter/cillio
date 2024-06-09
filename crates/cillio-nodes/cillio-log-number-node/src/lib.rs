#[allow(warnings)]
mod bindings;

use bindings::{Error, Guest, Inputs};
use chrono::prelude::*;

struct Component;

impl Guest for Component {
    fn process(in_: Inputs) -> Option<Error> {
        let current_time = Local::now();
        println!(
            "[{}] {}",
            current_time.format("%Y-%m-%d %H:%M:%S"),
            in_.number
        );
        None
    }
}

bindings::export!(Component with_types_in bindings);
