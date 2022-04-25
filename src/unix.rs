use core::fmt::{self, Formatter};
use core::mem::{MaybeUninit, transmute};
use core::slice::{self};
use core::str::{self};
use errno_sys::errno_location;
use iconv_sys::*;
use libc::{CODESET, E2BIG, c_char, c_int, nl_langinfo, strlen};

extern "C" {
    fn strerror(errnum: c_int) -> *mut c_char;
}

fn write_byte(f: &mut Formatter, c: u8) -> fmt::Result {
    write!(f, "\\x{:02x}", c)
}

fn write_fallback(f: &mut Formatter, s: &[u8]) -> fmt::Result {
    for &c in s {
        write_byte(f, c)?;
    }
    Ok(())
}

fn write_utf8_lossy(f: &mut Formatter, mut s: &[u8]) -> fmt::Result {
    loop {
        match str::from_utf8(s) {
            Ok(valid) => {
                write!(f, "{}", valid)?;
                break Ok(());
            }
            Err(error) => {
                let (valid, after_valid) = s.split_at(error.valid_up_to());
                let valid = unsafe { str::from_utf8_unchecked(valid) };
                write!(f, "{}", valid)?;
                let invalid_len = error.error_len().unwrap_or(after_valid.len());
                let (invalid, tail) = after_valid.split_at(invalid_len);
                write_fallback(f, invalid)?;
                s = tail;
            }
        }
    }
}

struct Iconv(iconv_t);

impl Drop for Iconv {
    fn drop(&mut self) {
        let iconv_close_res = unsafe { iconv_close(self.0) };
        debug_assert_eq!(iconv_close_res, 0);
    }
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    let msg = unsafe {
        let msg = strerror(e) as *const c_char;
        slice::from_raw_parts(msg as *const u8, strlen(msg))
    };
    let nl = unsafe {
        let nl = nl_langinfo(CODESET) as *const c_char;
        slice::from_raw_parts(nl as *const u8, strlen(nl) + 1)
    };
    if nl == b"UTF-8\0" {
        return write_utf8_lossy(f, msg);
    }
    let c = unsafe { iconv_open(b"UTF-8\0".as_ptr() as _, nl.as_ptr() as _) };
    if c == iconv_t::ERROR {
        return write_fallback(f, msg);
    }
    let c = Iconv(c);
    let mut msg_ptr = msg.as_ptr() as *const c_char as *mut c_char;
    let mut msg_len = msg.len();
    let mut uni_buf: [MaybeUninit<u8>; 128] = unsafe { MaybeUninit::uninit().assume_init() };
    loop {
        let mut uni_buf_ptr = uni_buf.as_mut_ptr() as *mut c_char;
        let mut uni_buf_len = uni_buf.len();
        let iconv_res: isize = unsafe { transmute(iconv(
            c.0,
            (&mut msg_ptr) as *mut _,
            (&mut msg_len) as *mut _,
            (&mut uni_buf_ptr) as *mut _,
            (&mut uni_buf_len) as *mut _
        )) };
        if iconv_res == -1 && errno_raw() != E2BIG {
            debug_assert!(msg_len > 0);
            write_byte(f, msg[msg.len() - msg_len])?;
            msg_ptr = unsafe { msg_ptr.add(1) };
            msg_len -= 1;
        } else {
            let uni_len = uni_buf.len() - uni_buf_len;
            let uni = &uni_buf[.. uni_len];
            let uni = unsafe { str::from_utf8_unchecked(transmute(uni)) };
            write!(f, "{}", uni)?;
            if iconv_res != -1 {
                debug_assert_eq!(msg_len, 0);
                return Ok(());
            }
        }
    }
}

pub fn errno_raw() -> i32 { 
    (unsafe { *errno_location() }) as i32
}

pub fn set_errno_raw(e: i32) {
    unsafe {
        *errno_location() = e;
    }
}