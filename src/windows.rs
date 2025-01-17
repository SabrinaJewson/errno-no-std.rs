use core::fmt::{self, Formatter};
use core::ptr::{null, null_mut};
use core::slice::{self};
use widestring::UStr;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::shared::ntdef::{LANG_NEUTRAL, LPWSTR, MAKELANGID, SUBLANG_DEFAULT};
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::*;

struct Buf(LPWSTR);

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0 as LPVOID) };
    }
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    let mut buf: LPWSTR = null_mut();
    let msg_len = unsafe { FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        null(),
        e as DWORD,
        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as _,
        &mut buf as *mut _ as LPWSTR,
        0,
        null_mut()
    ) };
    if msg_len == 0 {
        return write!(f, "error 0x{:04x}", e as DWORD);
    }
    let buf = Buf(buf);
    let msg = unsafe { slice::from_raw_parts(buf.0, msg_len as usize) };
    let trim = msg.iter().copied().rev().take_while(|&w| w == b'\r' as u16 || w == b'\n' as u16).count();
    let msg = UStr::from_slice(&msg[.. msg.len() - trim]);
    write!(f, "{}", msg.display())
}

pub fn errno_raw() -> i32 { 
    (unsafe { GetLastError() }) as i32
}

pub fn set_errno_raw(e: i32) {
    unsafe { SetLastError(e as DWORD) }
}
