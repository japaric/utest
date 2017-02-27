#![feature(compiler_builtins_lib)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![no_std]

extern crate compiler_builtins;
#[macro_use]
extern crate sc;

use core::intrinsics;

#[macro_use]
mod macros;

mod lang_items;
mod io;

#[no_mangle]
pub fn __test_start(ntests: usize) {
    eprintln!("running {} tests", ntests)
}

#[no_mangle]
pub fn __test_ignored(name: &str) {
    eprintln!("test {} ... ignored", name);
}

#[no_mangle]
pub fn __test_before_run(name: &str) {
    eprint!("test {} ... ", name);
}

#[no_mangle]
pub fn __test_panic_fmt(args: ::core::fmt::Arguments,
                        file: &'static str,
                        line: u32) {
    eprint!("\npanicked at '");
    io::write_fmt(args);
    eprintln!("', {}:{}", file, line);
}

#[no_mangle]
pub fn __test_failed(_name: &str) {
    eprintln!("FAILED");
}

#[no_mangle]
pub fn __test_success(_name: &str) {
    eprintln!("OK");
}

#[no_mangle]
pub fn __test_summary(passed: usize, failed: usize, ignored: usize) {
    eprintln!("\ntest result: {}. {} passed; {} failed; {} ignored",
              if failed == 0 { "OK" } else { "FAILED" },
              passed,
              failed,
              ignored);

    if failed != 0 {
        exit(101);
    }
}

/// Entry point
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }

    main(0, ::core::ptr::null());

    exit(0);
}

fn exit(code: i32) -> ! {
    unsafe {
        syscall!(EXIT, code as usize);

        intrinsics::unreachable()
    }
}

// Unused symbols related to unwinding that LLVM requires
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}

#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr1() {}
