//! Native Rust runner for the exact fabricated-native.ll function, using a
//! real mmap-backed allocator placement permitted by Linux mmap_min_addr.
use std::ffi::c_void;
unsafe extern "C" {
    fn phage_target(n: u8) -> bool;
    fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, off: isize) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
    assert!(size <= 64 && align == 1);
    // SAFETY: request a fresh page without replacing mappings; no pointer is
    // accessed unless mmap succeeds at exactly the requested address.
    let result = unsafe { mmap(0x100000usize as *mut c_void, 4096, 3, 0x100022, -1, 0) };
    assert_eq!(result as usize, 0x100000, "controlled native placement unavailable");
    result.cast()
}
fn main() {
    // SAFETY: the LLVM function only computes addresses and calls the valid
    // mmap allocator hook above. It never dereferences its fabricated pointer.
    let result = unsafe { phage_target(32) };
    assert!(!result, "the fabricated pointer must not reserve the middle of this allocation");
    // SAFETY: release exactly the successfully mapped page after the call.
    assert_eq!(unsafe { munmap(0x100000usize as *mut c_void, 4096) }, 0);
    println!("confirmed: allocation spans fabricated address 1048592; property returned false");
}
