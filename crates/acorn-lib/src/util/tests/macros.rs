use alloc::sync::Arc;
use std::io::{Result, Write};
use std::sync::Mutex;
use tracing_subscriber::fmt::MakeWriter;

struct SharedWriter(Arc<Mutex<Vec<u8>>>);
struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);
impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        SharedWriterGuard(Arc::clone(&self.0))
    }
}
impl Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        match self.0.lock() {
            | Ok(mut logs) => {
                logs.extend_from_slice(buffer);
                Ok(buffer.len())
            }
            | Err(why) => Err(std::io::Error::other(why.to_string())),
        }
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
fn captured_logs(operation: impl FnOnce()) -> String {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(SharedWriter(Arc::clone(&logs)))
        .finish();
    tracing::subscriber::with_default(subscriber, operation);
    logs.lock().map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).unwrap_or_default()
}

#[test]
fn test_fail_formats_messages() {
    let why = "because";
    let logs = captured_logs(|| {
        crate::fail!("Whoops - {why}");
        crate::fail!("Plain failure");
        crate::fail!("Legacy - {}", why);
    });
    assert!(logs.contains("Whoops - because"));
    assert!(logs.contains("Plain failure"));
    assert!(logs.contains("Legacy - because"));
    assert!(!logs.contains("{why}"));
}
