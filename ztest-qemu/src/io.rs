use core::{fmt, slice};
use core::fmt::Write;

/// STDERR File descriptor
const STDERR: usize = 2;

struct Stderr;

impl Stderr {
    fn write_all(&mut self, mut buffer: &[u8]) {
        while !buffer.is_empty() {
            match unsafe {
                      syscall!(WRITE, STDERR, buffer.as_ptr(), buffer.len()) as
                      isize
                  } {
                n if n >= 0 => {
                    buffer =
                        unsafe {
                            slice::from_raw_parts(buffer.as_ptr().offset(n),
                                                  buffer.len() - n as usize)
                        }
                }
                // ignore errors
                _ => return,
            }
        }
    }
}

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes());
        Ok(())
    }
}

pub fn write_fmt(args: fmt::Arguments) {
    Stderr.write_fmt(args).ok();
}

pub fn write_str(string: &str) {
    Stderr.write_all(string.as_bytes())
}
