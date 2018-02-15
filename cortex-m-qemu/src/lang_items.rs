use core::intrinsics;

#[lang = "start"]
extern "C" fn start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination,
{
    main();

    0
}

#[lang = "termination"]
pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}


#[lang = "panic_fmt"]
#[no_mangle]
unsafe extern "C" fn panic_fmt(args: ::core::fmt::Arguments,
                               file: &'static str,
                               line: u32)
                               -> ! {
    #[allow(improper_ctypes)]
    extern "Rust" {
        fn __test_panic_fmt(args: ::core::fmt::Arguments,
                            file: &'static str,
                            line: u32);
    }

    __test_panic_fmt(args, file, line);

    intrinsics::abort()
}
