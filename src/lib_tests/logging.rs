// src/lib_tests/logging.rs
// Tests for safe logging helpers

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
        }
    }

    #[test]
    fn write_log_line_ignores_write_errors() {
        let mut out = FailingWriter;
        crate::write_log_line(&mut out, "test");
    }
}
