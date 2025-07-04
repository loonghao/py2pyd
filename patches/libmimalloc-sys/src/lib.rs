// No-op replacement for libmimalloc-sys
// This prevents cross-compilation issues with MinGW

// Re-export the minimal interface that mimalloc expects
pub use std::os::raw::{c_char, c_int, c_void};

// Provide empty implementations of the functions that mimalloc might call
#[no_mangle]
pub extern "C" fn mi_malloc(_size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn mi_free(_ptr: *mut c_void) {
    // No-op
}

#[no_mangle]
pub extern "C" fn mi_realloc(_ptr: *mut c_void, _size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn mi_calloc(_count: usize, _size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn mi_malloc_aligned(_size: usize, _alignment: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn mi_zalloc_aligned(_size: usize, _alignment: usize) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn mi_realloc_aligned(_ptr: *mut c_void, _size: usize, _alignment: usize) -> *mut c_void {
    std::ptr::null_mut()
}