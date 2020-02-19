macro_rules! eprint {
    ($s:expr) => ($crate::io::write_str($s));
    ($($arg:tt)*) => ($crate::io::write_fmt(format_args!($($arg)*)));
}

macro_rules! eprintln {
    () => (eprint!("\n"));
    ($fmt:expr) => (eprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (eprint!(concat!($fmt, "\n"), $($arg)*));
}
