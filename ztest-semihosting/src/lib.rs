#![feature(asm)]
#![no_std]

#[macro_use]
extern crate cortex_m_semihosting;

#[no_mangle]
pub fn __test_start(ntests: usize) {
    ehprintln!("running {} tests", ntests)
}

#[no_mangle]
pub fn __test_ignored(name: &str) {
    ehprintln!("test {} ... ignored", name);
}

#[no_mangle]
pub fn __test_before_run(name: &str) {
    ehprint!("test {} ... ", name);
}

#[no_mangle]
pub fn __test_panic_fmt(args: ::core::fmt::Arguments,
                        file: &'static str,
                        line: u32) {
    ehprint!("\npanicked at '");
    cortex_m_semihosting::io::ewrite_fmt(args);
    ehprintln!("', {}:{}", file, line);
}

#[no_mangle]
pub fn __test_failed(_name: &str) {
    ehprintln!("FAILED");
}

#[no_mangle]
pub fn __test_success(_name: &str) {
    ehprintln!("OK");
}

#[no_mangle]
pub fn __test_summary(passed: usize, failed: usize, ignored: usize) {
    ehprintln!("\ntest result: {}. {} passed; {} failed; {} ignored",
               if failed == 0 { "OK" } else { "FAILED" },
               passed,
               failed,
               ignored);

    unsafe {
        asm!("bkpt" :::: "volatile");
    }
}
