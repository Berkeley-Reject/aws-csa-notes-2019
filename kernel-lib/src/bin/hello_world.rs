#![no_std]
#![no_main]

use log::info;

extern crate kernel_lib;

#[no_mangle]
fn main() -> i32 {
    info!("hello, world!");
    0
}
