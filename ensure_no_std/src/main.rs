#![feature(default_alloc_error_handler)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::alloc::Layout;
use core::panic::PanicInfo;
use errno_no_std::*;
#[cfg(not(windows))]
use libc::EINVAL;
#[cfg(not(windows))]
use libc::exit;
use libc_alloc::LibcAlloc;
#[cfg(windows)]
use winapi::shared::minwindef::UINT;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;
#[cfg(windows)]
use winapi::shared::winerror::ERROR_ACCESS_DENIED;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    ExitProcess(code);
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(99) }
}

#[no_mangle]
pub fn rust_oom(_layout: Layout) -> ! {
    unsafe { exit(98) }
}

#[cfg(not(windows))]
const ERR: i32 = EINVAL;

#[cfg(windows)]
const ERR: i32 = ERROR_ACCESS_DENIED as i32;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    set_errno(Errno(ERR));
    assert_eq!(errno().0, ERR);
    0
}