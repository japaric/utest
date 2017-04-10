#![no_std]

#[macro_export]
macro_rules! upanic {
    () => (
        upanic!("explicit panic")
    );
    ($fmt:expr) => ({
        upanic!($fmt,)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        #[allow(improper_ctypes)]
        extern "Rust" {
            static mut __TEST_PANICKED: bool;
            fn __test_panic_fmt(args: ::core::fmt::Arguments,
                                file: &'static str,
                                line: u32);
        }
        unsafe {
            __TEST_PANICKED = true;
            __test_panic_fmt(format_args!($fmt, $($arg)*), file!(), line!());
            return
        }
    });
}
