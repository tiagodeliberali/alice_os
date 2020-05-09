#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(alice_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use alice_os::println;
use core::panic::PanicInfo;

#[cfg(test)]
use alice_os::{serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Alice OS");
    println!("--------");
    println!("version: {}", 0.1);

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    alice_os::test_panic_handler(info);
    loop {}
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}
