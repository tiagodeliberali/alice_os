#![no_std]
#![no_main]

use core::panic::PanicInfo;

static CONTENT: &[u8] = b"HELLO ALICE";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0x8000 as *mut u8;

    for (i, &byte) in CONTENT.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
