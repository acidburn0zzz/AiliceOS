#![no_std]
#![no_main]
#![feature(lang_items)]

#[macro_use]
pub mod console;

pub mod lang;



use bootloader::{entry_point, BootInfo};


entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    println!("I'm from Kernel!");
    loop {}
}
