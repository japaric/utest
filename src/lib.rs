#![no_std]

#[doc(hidden)]

// TODO mark `#[used]` and hide (remove `pub`)
#[no_mangle]
pub static mut __TEST_PANICKED: bool = false;

pub fn test_main_static(tests: &[&TestDescAndFn]) {
    #[allow(improper_ctypes)]
    extern "Rust" {
        fn __test_before_run(name: &str);
        fn __test_failed(name: &str);
        fn __test_ignored(name: &str);
        fn __test_start(ntests: usize);
        fn __test_success(name: &str);
        fn __test_summary(passed: usize, failed: usize, ignored: usize);
    }

    unsafe {
        __test_start(tests.len());

        let mut failed = 0;
        let mut ignored = 0;
        let mut passed = 0;
        for test in tests {
            if test.desc.ignore {
                ignored += 1;
                __test_ignored(test.desc.name.0);
            } else {
                __test_before_run(test.desc.name.0);

                __TEST_PANICKED = false;

                test.testfn.0();

                if __TEST_PANICKED ==
                   (test.desc.should_panic == ShouldPanic::Yes) {
                    passed += 1;
                    __test_success(test.desc.name.0);
                } else {
                    failed += 1;
                    __test_failed(test.desc.name.0);
                }
            }

        }

        __test_summary(passed, failed, ignored);
    }
}

pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

pub fn assert_test_result<T: Termination>(result: T) {
    assert_eq!(result.report(), 0);
}

// required for compatibility with the `rustc --test` interface
pub struct TestDescAndFn {
    pub desc: TestDesc,
    pub testfn: StaticTestFn,
}

pub struct TestDesc {
    pub allow_fail: bool,
    pub ignore: bool,
    pub name: StaticTestName,
    pub should_panic: ShouldPanic,
    pub test_type: TestType,
}

pub struct StaticTestName(pub &'static str);
pub struct StaticTestFn(pub fn());

#[derive(PartialEq)]
pub enum ShouldPanic {
    No,
    Yes,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestType {
    UnitTest,
    IntegrationTest,
    DocTest,
    Unknown,
}
