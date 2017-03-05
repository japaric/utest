# `μtest`

> Unit `#[test]`ing for microcontrollers and other `no_std` systems

![Running unit tests on a Cortex-M3 microcontroller](https://i.imgur.com/19gN3tT.png)

**WARNING** This crate relies on `#[test]` / `rustc` implementation details and
could break at any time.

# Table of Contents

- [Features](#features)

- [Limitations](#limitations)

- [Testing on an emulated Cortex-M processor](#testing-on-an-emulated-cortex-m-processor)

- [Testing on a real Cortex-M microcontroller](#testing-on-a-real-cortex-m-microcontroller)
  - [Requirements](#requirements)
  - [Steps](#steps)

- [Building a custom test runner](#building-a-custom-test-runner)
  - [Hooks](#hooks)
    - [`__test_start`](#__test_start)
    - [`__test_ignored`](#__test_ignored)
    - [`__test_before_run`](#__test_before_run)
    - [`__test_failed`](#__test_failed)
    - [`__test_success`](#__test_success)
    - [`__test_summary`](#__test_summary)
    - [`__test_panic_fmt`](#__test_panic_fmt)

- [How does this work without unwinding?](#how-does-this-work-without-unwinding)

- [License](#license)
  - [Contribution](#contribution)

# Features

- Doesn't depend on `std`.

- Fully configurable, through [hooks](#hooks).

# Limitations

- Tests are executed sequentially. This is required to support bare metal
  systems where threads may not be implemented.

- All tests will print to stdout / stderr as they progress.

- `panic!`s *outside* the crate under test will NOT mark the unit test as
  failed; those `panic`s will likely abort the test runner but this is
  implementation defined. (more about this later)

- `#[bench]` is not supported.

- No colorized output

# Testing on an emulated Cortex-M processor

Using the `utest-cortex-m-qemu` test runner.

This uses QEMU user emulation to emulate a Cortex-M processor that has access to
the host Linux kernel thus you can do stuff like using the WRITE system call to
print to the host console.

The downside of this approach is that the QEMU user emulation doesn't emulate
the peripherals of a Cortex-M microcontroller so this is mainly useful to test
pure functions / functions that don't do embedded I/O (by embedded I/O, I mean
I2C, Serial, PWM, etc.).

0) Start with a `no_std` library crate.

```
$ cargo new --lib foo && cd $_

$ edit src/lib.rs && cat $_
```

``` rust
#![no_std]

#[test]
fn assert() {
    assert!(true);
}

#[test]
fn assert_failed() {
    assert!(false, "oh noes");
}

#[test]
fn assert_eq() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn assert_eq_failed() {
    let answer = 24;
    assert_eq!(answer, 42, "The answer was 42!");
}

#[test]
fn it_works() {}

#[test]
#[should_panic]
fn should_panic() {
    panic!("Let's panic!")
}
```

1) Append this to your crate's Cargo.toml

``` toml
[target.thumbv7m-linux-eabi.dev-dependencies.utest-macros]
git = "https://github.com/japaric/utest"

[target.thumbv7m-linux-eabi.dev-dependencies.test]
git = "https://github.com/japaric/utest"

[target.thumbv7m-linux-eabi.dev-dependencies.utest-cortex-m-qemu]
git = "https://github.com/japaric/utest"
```

**NOTE** Change `thumbv7m-linux-eabi` as necessary. The other options are
`thumbv6m-linux-eabi`, `thumbv7em-linux-eabi` and `thumbv7em-linux-eabihf`.
(Yes, `linux` not `none`)

2) Add this to your `src/lib.rs`

``` rust
// test runner
#[cfg(all(target_arch = "arm",
          not(all(target_env = "gnu", target_env = "musl")),
          target_os = "linux",
          test))]
extern crate utest_cortex_m_qemu;

// overrides `panic!`
#[cfg(all(target_arch = "arm",
          not(all(target_env = "gnu", target_env = "musl")),
          target_os = "linux",
          test))]
#[macro_use]
extern crate utest_macros;
```

3) Create the target specification file.

Start with this target specification

```
$ rustc \
  -Z unstable-options \
  --print target-spec-json \
  --target thumbv7m-none-eabi \
  | tee thumbv7m-linux-eabi.json
```

``` js
{
  "abi-blacklist": [
    "stdcall",
    "fastcall",
    "vectorcall",
    "win64",
    "sysv64"
  ],
  "arch": "arm",
  "data-layout": "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64",
  "env": "",
  "executables": true,
  "is-builtin": true,
  "linker": "arm-none-eabi-gcc",
  "llvm-target": "thumbv7m-none-eabi",
  "max-atomic-width": 32,
  "os": "none",
  "panic-strategy": "abort",
  "relocation-model": "static",
  "target-endian": "little",
  "target-pointer-width": "32",
  "vendor": ""
}
```

And perform these modifications

``` diff
     "data-layout": "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64",
     "env": "",
     "executables": true,
-    "is-builtin": true,
     "linker": "arm-none-eabi-gcc",
     "llvm-target": "thumbv7m-none-eabi",
     "max-atomic-width": 32,
-    "os": "none",
+    "os": "linux",
     "panic-strategy": "abort",
+    "pre-link-args": ["-nostartfiles"],
     "relocation-model": "static",
     "target-endian": "little",
     "target-pointer-width": "32",
```

4) Built the test runner

```
$ export RUST_TARGET_PATH=$(pwd)

$ xargo test --target thumbv7m-linux-eabi --no-run
```

5) Execute the test runner using QEMU

```
$ qemu-arm target/thumbv7m-linux-eabi/debug/deps/foo-aacd724200d968b7
running 6 tests
test assert ... OK
test assert_failed ...
panicked at 'oh noes', src/lib.rs:23
FAILED
test assert_eq ... OK
test assert_eq_failed ...
panicked at 'assertion failed: `(left == right)` (left: `24`, right: `42`): The answer was 42!', src/lib.rs:34
FAILED
test it_works ... OK
test should_panic ...
panicked at 'Let's panic!', src/lib.rs:43
OK
```

# Testing on a real Cortex-M microcontroller

Using the `utest-cortex-m-semihosting` test runner.

## Requirements

- Your target crate must support vanilla `fn main()`. This means that the
  `start` lang item must be defined somewhere in your crate dependency graph.

- The `panic_fmt` lang item must be defined in your crate dependency graph.
  Hitting `panic_fmt` while running the test suite is considered a fatal error
  so it doesn't matter how you have implemented it.

These two requirements can be fulfilled if your crate is based on the
[`cortex-m-template`](https://github.com/japaric/cortex-m-template).

- You should be able to use GDB to run / debug a Rust program. GDB is required
  for semihosting.

## Steps

0) Start with a crate that meets the requirements and some unit tests.

```
$ cargo new vl --template https://github.com/japaric/cortex-m-template

$ cd vl

$ edit memory.x && head $_
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

$ edit tests/foo.rs && cat $_
```

``` rust
#![no_std]

extern crate vl;

use core::ptr;

#[test]
fn assert() {
    assert!(true);
}

#[test]
fn assert_failed() {
    assert!(false, "oh noes");
}

#[test]
fn assert_eq() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn assert_eq_failed() {
    let answer = 24;
    assert_eq!(answer, 42, "The answer was 42!");
}

// STM32F103xx = medium density device -> DEVICE_ID = 0x410
// See section 31.6.1 of the reference manual
// (http://www.st.com/resource/en/reference_manual/cd00171190.pdf)
#[test]
fn device_id() {
    assert_eq!(unsafe { ptr::read_volatile(0xe004_2000 as *const u32) } &
               ((1 << 12) - 1),
               0x410);
}

#[ignore]
#[test]
fn ignored() {}

#[test]
#[should_panic]
fn should_panic() {
    panic!("Let's panic!")
}
```

1) Append this to your Cargo.toml

``` toml
[target.thumbv7m-none-eabi.dev-dependencies.test]
git = "https://github.com/japaric/utest"

[target.thumbv7m-none-eabi.dev-dependencies.utest-macros]
git = "https://github.com/japaric/utest"

[target.thumbv7m-none-eabi.dev-dependencies.utest-cortex-m-semihosting]
git = "https://github.com/japaric/utest"
```

2) Add this to you your integration test file (`tests/foo.rs` as per our
   example)

``` rust
#[cfg(all(target_arch = "arm",
          not(all(target_env = "gnu", target_env = "musl"))))]
#[macro_use]
extern crate utest_macros;

#[cfg(all(target_arch = "arm",
          not(all(target_env = "gnu", target_env = "musl"))))]
extern crate utest_cortex_m_semihosting;
```

3) If required (this is required for `cortex-m-template` based crates), define
   how exceptions and interrupts are handled. In our example, add this to
   `tests/foo.rs`.

``` rust
use vl::exceptions::{self, Exceptions};
use vl::interrupts::{self, Interrupts};

#[no_mangle]
pub static _EXCEPTIONS: Exceptions =
    Exceptions { ..exceptions::DEFAULT_HANDLERS };

#[no_mangle]
pub static _INTERRUPTS: Interrupts =
    Interrupts { ..interrupts::DEFAULT_HANDLERS };
```

4) Build the test runner

```
$ xargo test --target thumbv7m-none-eabi --test foo --no-run
```

5) Flash the test runner and execute the program using GDB.

**NOTE** These steps assume OpenOCD support.

If testing a crate based on the `cortex-m-template`, you'll only have to launch
OpenOCD.

```
# Terminal 1
$ openocd -f interface/stlink-v1.cfg -f target/stm32f1x.cfg
```

and then launch GDB.

```
# Terminal 2
$ arm-none-eabi-gdb ./target/thumbv7m-none-eabi/debug/foo-87b629153685d76f

(gdb) continue
```

You should see this in the OpenOCD output

```
# Terminal 1
running 7 tests
test assert ... OK
test assert_failed ...
panicked at 'oh noes', tests/foo.rs:26
FAILED
test assert_eq ... OK
test assert_eq_failed ...
panicked at 'assertion failed: `(left == right)` (left: `24`, right: `42`): The answer was 42!', tests/foo.rs:37
FAILED
test device_id ... OK
test ignored ... ignored
test should_panic ...
panicked at 'Let's panic!', tests/foo.rs:57
OK

test result: FAILED. 4 passed; 2 failed; 1 ignored
```

If you are not using a `cortex-m-template` based crate, then make sure you
enable semihosting from the GDB command line.

```
(gdb) monitor arm semihosting enable
```

# Building a custom test runner

You can create a custom test runner that, for example, doesn't require executing
the test runner under GDB and that instead reports the tests results via Serial
port or ITM.

The best way to implement a custom test runner is to base your implementation on
the implementation of the two tests runners shown above.

- [`utest-cortex-m-qemu`](/cortex-m-qemu)

- [`utest-cortex-m-semihosting`](/cortex-m-semihosting)

But in a nutshell you'll have to define all these "hook" functions:

## Hooks

Hooks are just vanilla functions with predefined symbol names that configure the
behavior of the test runner.

#### `__test_start`

Runs before the unit tests are executed.

Signature:

``` rust
/// `ntests`, number of unit tests (functions marked with `#[test]`)
#[no_mangle]
pub fn __test_start(ntests: usize) {
    ..
}
```

#### `__test_ignored`

Runs when a test if marked as `#[ignore]`d.

Signature:

``` rust
/// `name`, name of the ignored test
#[no_mangle]
pub fn __test_ignored(name: &'static str) {
    ..
}
```

#### `__test_before_run`

Runs right before an unit test gets executed.

Signature:

``` rust
/// `name`, name of the test that's about to be executed
#[no_mangle]
pub fn __test_before_run(name: &'static str) {
    ..
}
```

#### `__test_failed`

Runs if the unit test failed

Signature:

``` rust
/// `name`, name of the test that failed
#[no_mangle]
pub fn __test_failed(name: &'static str) {
    ..
}
```

#### `__test_success`

Runs if the unit test succeeded

Signature:

``` rust
/// `name`, name of the test that "passed"
#[no_mangle]
pub fn __test_success(name: &'static str) {
    ..
}
```

#### `__test_summary`

Runs after all the unit tests have been executed.

Signature:

``` rust
/// `passed`, number of unit tests that passed
/// `failed`, number of unit tests that failed
/// `ignored`, number of unit tests that were ignored
#[no_mangle]
pub fn __test_summary(passed: usize, failed: usize, ignored: usize) {
    ..
}
```

#### `__test_panic_fmt`

Runs when `utest-macros`'s `panic!` macro is called.

Signature:

``` rust
/// Signature matches the signature of the `panic_fmt` lang item
#[no_mangle]
pub fn __test_panic_fmt(args: ::core::fmt::Arguments,
                        file: &'static str,
                        line: u32) {
    ..
}
```

# How does this work without unwinding?

`std` unit tests rely heavily on unwinding. Each unit test is run inside a
`catch_unwind` block and if the unit test panics then the panic is caught and
the test is marked as failed (or as passed if the unit test was marked with
`#[should_panic]`).

We attempt to emulate this behavior by overriding the `panic!` macro to mark the
test failed and then early `return` instead of unwind. Of course, this emulation
breaks down if the `panic!` originates from outside the crate under test,
because `panic` is not overridden in that scope. So this setup is certainly not
perfect.

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
