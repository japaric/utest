use core::intrinsics;

#[lang = "start"]
extern "C" fn start(main: fn(),
                    _argc: isize,
                    _argv: *const *const u8)
                    -> isize {
    main();

    0
}

#[lang = "panic_fmt"]
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
