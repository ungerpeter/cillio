#[allow(warnings)]
mod bindings;

use bindings::Guest;
use chrono::prelude::*;

struct Component;

impl Guest for Component {
    fn run(number: f32) {
        let current_time = Local::now();
        println!("[{}] {}", current_time.format("%Y-%m-%d %H:%M:%S"), number);
    }
}

bindings::export!(Component with_types_in bindings);
