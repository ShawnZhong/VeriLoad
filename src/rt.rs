use core::arch::asm;
use std::ffi::{c_void, CString};

pub const PROT_READ: i32 = 0x1;
pub const PROT_WRITE: i32 = 0x2;
pub const PROT_EXEC: i32 = 0x4;

const MAP_PRIVATE: i32 = 0x02;
const MAP_ANONYMOUS: i32 = 0x20;
const O_RDONLY: i32 = 0;
const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
    fn open(path: *const i8, flags: i32) -> i32;
    fn pread(fd: i32, buf: *mut c_void, count: usize, offset: i64) -> isize;
    fn close(fd: i32) -> i32;
}

pub fn fatal(message: impl AsRef<str>) -> ! {
    let msg = message.as_ref();
    eprintln!("FATAL: {msg}");
    panic!("FATAL: {msg}");
}

pub fn log(message: impl AsRef<str>) {
    eprintln!("[veriload] {}", message.as_ref());
}

pub fn checked_usize_from_u64(value: u64, context: &str) -> usize {
    usize::try_from(value).unwrap_or_else(|_| {
        fatal(format!(
            "u64 to usize conversion overflow for {context}: {value}"
        ))
    })
}

pub fn checked_i64_from_u64(value: u64, context: &str) -> i64 {
    i64::try_from(value).unwrap_or_else(|_| {
        fatal(format!(
            "u64 to i64 conversion overflow for {context}: {value}"
        ))
    })
}

fn last_os_error(prefix: &str) -> String {
    let err = std::io::Error::last_os_error();
    format!("{prefix}: {err}")
}

pub fn rt_open_read(path: &str) -> i32 {
    let c_path = CString::new(path).unwrap_or_else(|_| {
        fatal(format!("path contains interior NUL byte: {path}"));
    });

    let fd = unsafe { open(c_path.as_ptr(), O_RDONLY) };
    if fd < 0 {
        fatal(last_os_error(&format!("open failed for {path}")));
    }
    fd
}

pub fn rt_pread_exact(fd: i32, file_offset: u64, buf: &mut [u8]) {
    let mut done = 0usize;
    while done < buf.len() {
        let off = file_offset
            .checked_add(done as u64)
            .unwrap_or_else(|| fatal("pread offset overflow"));
        let off_i64 = checked_i64_from_u64(off, "pread offset");

        let rc = unsafe {
            pread(
                fd,
                buf[done..].as_mut_ptr() as *mut c_void,
                buf.len() - done,
                off_i64,
            )
        };

        if rc < 0 {
            fatal(last_os_error("pread failed"));
        }
        if rc == 0 {
            fatal(format!(
                "pread reached EOF early: fd={fd} requested={} read={done}",
                buf.len()
            ));
        }

        done = done
            .checked_add(rc as usize)
            .unwrap_or_else(|| fatal("pread byte count overflow"));
    }
}

pub fn rt_close(fd: i32) {
    let rc = unsafe { close(fd) };
    if rc != 0 {
        fatal(last_os_error(&format!("close failed for fd={fd}")));
    }
}

pub fn rt_read_file(path: &str) -> Vec<u8> {
    let meta = std::fs::metadata(path)
        .unwrap_or_else(|e| fatal(format!("metadata failed for {path}: {e}")));
    let len = checked_usize_from_u64(meta.len(), "file length");

    let fd = rt_open_read(path);
    let mut out = vec![0u8; len];
    if len > 0 {
        rt_pread_exact(fd, 0, &mut out);
    }
    rt_close(fd);
    out
}

pub fn rt_mmap_rw(len: usize) -> *mut u8 {
    if len == 0 {
        fatal("rt_mmap_rw called with len=0");
    }

    let ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if ptr == MAP_FAILED {
        fatal(last_os_error(&format!("mmap failed for len={len}")));
    }

    ptr as *mut u8
}

pub fn rt_mprotect(addr: *mut u8, len: usize, prot: i32, what: &str) {
    if len == 0 {
        return;
    }

    let rc = unsafe { mprotect(addr as *mut c_void, len, prot) };
    if rc != 0 {
        fatal(last_os_error(&format!(
            "mprotect failed for {what} addr=0x{:x} len={} prot=0x{:x}",
            addr as usize, len, prot
        )));
    }
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn enter(entry: u64, sp: *const u8) -> ! {
    asm!(
        "mov rsp, {stack}",
        "jmp {target}",
        stack = in(reg) sp,
        target = in(reg) entry,
        options(noreturn)
    );
}

#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn enter(_entry: u64, _sp: *const u8) -> ! {
    fatal("enter is only implemented for x86_64");
}
