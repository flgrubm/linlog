// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

/// The stack of a thread that reads and parses the input: a main thread's,
/// on which the same work runs when there is no time limit.
const LOAD_STACK: usize = 8 << 20;

/// A time limit as a flag: a thread of its own raises it once the limit
/// has passed, so that asking whether it has is one load and no look at a
/// clock. A search polls its stop condition anywhere from millions of
/// times a second to once in many milliseconds, and a condition that
/// looks at the clock only every so many polls is late by that many: on a
/// large sequent by half a minute.
pub(crate) struct Deadline {
    /// Whether the limit has passed.
    passed: Arc<AtomicBool>,
    /// The limit and the moment it ends, if there is one.
    limit: Option<(Duration, Instant)>,
    /// Dropped with the deadline, which wakes the timer thread before its
    /// time so that it ends.
    _alive: Option<Sender<()>>,
}

impl Deadline {
    /// Starts a limit counted from `start`; `None` is a deadline that
    /// never passes, and starts no thread.
    pub(crate) fn start(limit: Option<Duration>, start: Instant) -> Result<Self> {
        let passed = Arc::new(AtomicBool::new(false));
        let Some(limit) = limit else {
            return Ok(Self {
                passed,
                limit: None,
                _alive: None,
            });
        };
        let end = start + limit;
        let (alive, dropped) = channel::<()>();
        let flag = Arc::clone(&passed);
        thread::Builder::new()
            .name("time limit".into())
            .spawn(move || {
                let left = end.saturating_duration_since(Instant::now());
                // Nothing is ever sent: the wait ends at the limit, or
                // early when the deadline is dropped.
                if dropped.recv_timeout(left) == Err(RecvTimeoutError::Timeout) {
                    flag.store(true, Ordering::Relaxed);
                }
            })
            .context("cannot start the thread that keeps the time limit")?;
        Ok(Self {
            passed,
            limit: Some((limit, end)),
            _alive: Some(alive),
        })
    }

    /// Whether the limit has passed.
    pub(crate) fn passed(&self) -> bool {
        self.passed.load(Ordering::Relaxed)
    }

    /// The limit, if there is one.
    pub(crate) fn limit(&self) -> Option<Duration> {
        self.limit.map(|(limit, _)| limit)
    }

    /// Runs `load` and returns its result, or `None` when the limit passed
    /// first. Under a limit `load` runs on a thread of its own, which is
    /// left behind when the limit passes: reading a file and parsing it
    /// cannot be stopped from inside, so the command answers without them
    /// and ends, and the thread ends with the process.
    pub(crate) fn within<T: Send + 'static>(
        &self,
        load: impl FnOnce() -> T + Send + 'static,
    ) -> Result<Option<T>> {
        let Some((_, end)) = self.limit else {
            return Ok(Some(load()));
        };
        let (sender, receiver) = channel();
        let thread = thread::Builder::new()
            .name("load".into())
            .stack_size(LOAD_STACK)
            .spawn(move || {
                // The receiver is gone when the limit passed first.
                let _ = sender.send(load());
            })
            .context("cannot start the thread that reads the input")?;
        match receiver.recv_timeout(end.saturating_duration_since(Instant::now())) {
            Ok(loaded) => Ok(Some(loaded)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            // The thread ended without a result: it panicked.
            Err(RecvTimeoutError::Disconnected) => match thread.join() {
                Err(panic) => std::panic::resume_unwind(panic),
                Ok(()) => unreachable!("the thread sends before it ends"),
            },
        }
    }
}

/// A line on standard error while a search runs long, taken back when it
/// ends, so that a wait at a terminal is never silent and the output is
/// what it would be without it. Nothing is written when standard error is
/// not a terminal, nor for a search that ends in time.
pub(crate) struct Notice {
    /// Dropped when the search ends, which wakes the thread.
    running: Option<Sender<()>>,
    /// The thread that writes the line and takes it back.
    thread: Option<thread::JoinHandle<()>>,
}

impl Notice {
    /// Writes `line` on standard error once `after` has passed, until the
    /// notice is dropped.
    pub(crate) fn start(after: Duration, line: String) -> Self {
        use std::io::{IsTerminal, Write};
        if !std::io::stderr().is_terminal() {
            return Self {
                running: None,
                thread: None,
            };
        }
        let (running, ended) = channel::<()>();
        let thread = thread::Builder::new()
            .name("notice".into())
            .spawn(move || {
                // Nothing is ever sent: the waits end when the notice is
                // dropped.
                if ended.recv_timeout(after) != Err(RecvTimeoutError::Timeout) {
                    return;
                }
                let mut err = std::io::stderr().lock();
                let _ = write!(err, "{line}");
                let _ = err.flush();
                drop(err);
                let _ = ended.recv();
                // Back to the start of the line, which is then cleared.
                let mut err = std::io::stderr().lock();
                let _ = write!(err, "\r\x1b[2K");
                let _ = err.flush();
            })
            .ok();
        Self {
            running: Some(running),
            thread,
        }
    }
}

impl Drop for Notice {
    /// Ends the thread, which takes the line back if it wrote it, and
    /// waits for it, so that nothing is written after.
    fn drop(&mut self) {
        self.running.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
