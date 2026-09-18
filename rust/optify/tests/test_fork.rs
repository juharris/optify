#[cfg(unix)]
mod unix {
    use optify::{provider::OptionsRegistry, OptionsProvider};
    use std::{
        io,
        os::unix::process::ExitStatusExt,
        path::Path,
        process::ExitStatus,
        thread,
        time::{Duration, Instant},
    };

    /// Ensures a stalled child is killed and reaped even when the test fails.
    struct Child {
        pid: libc::pid_t,
        reaped: bool,
    }

    impl Child {
        fn wait(&mut self) -> io::Result<ExitStatus> {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let mut status = 0;
                let result = unsafe { libc::waitpid(self.pid, &mut status, libc::WNOHANG) };
                if result == self.pid {
                    self.reaped = true;
                    return Ok(ExitStatus::from_raw(status));
                }
                if result == -1 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::ECHILD) {
                        self.reaped = true;
                    }
                    if error.kind() != io::ErrorKind::Interrupted {
                        return Err(error);
                    }
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Provider build in forked child did not finish within 10 seconds",
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    impl Drop for Child {
        fn drop(&mut self) {
            if !self.reaped {
                unsafe {
                    libc::kill(self.pid, libc::SIGKILL);
                    while libc::waitpid(self.pid, std::ptr::null_mut(), 0) == -1 {
                        if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn build_and_verify() {
        let suite = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/test_suites/simple");
        let expectation: serde_json::Value = serde_json::from_slice(
            &std::fs::read(suite.join("expectations/aliases.json")).unwrap(),
        )
        .unwrap();
        let features: Vec<String> =
            serde_json::from_value(expectation["features"].clone()).unwrap();
        let provider = OptionsProvider::build(suite.join("configs")).unwrap();
        assert_eq!(
            provider.get_options("myConfig", &features).unwrap(),
            expectation["options"]["myConfig"],
        );
    }

    pub fn run() {
        build_and_verify();

        // Deliberately exercise fork without exec, as used by Ruby process workers.
        // This regression does not establish general safety of Rust code after fork.
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0, "fork failed: {}", io::Error::last_os_error());
        if pid == 0 {
            let success = std::panic::catch_unwind(build_and_verify).is_ok();
            // Avoid running inherited process cleanup in the child.
            unsafe { libc::_exit(if success { 0 } else { 1 }) };
        }

        let mut child = Child { pid, reaped: false };
        let status = child.wait().expect("Could not finish fork regression test");
        assert!(status.success(), "Forked provider build failed: {status}");
        println!("test_build_after_fork ... ok");
    }
}

fn main() {
    #[cfg(unix)]
    unix::run();
    #[cfg(not(unix))]
    println!("test_build_after_fork ... skipped (requires Unix fork)");
}
