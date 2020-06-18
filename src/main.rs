
use std::time::{Duration, Instant};

mod wlr;

use wlr::*;

use lazy_static::lazy_static;
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CursorMode {
    Passthrough,
    Move,
    Resize
}

fn main() {
    println!("Hello, world!");

    lazy_static::initialize(&START_TIME);

    unsafe {
        wlr_log_init(wlr_log_importance_WLR_DEBUG, None);

    }
}
