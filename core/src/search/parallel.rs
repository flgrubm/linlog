// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The parallel runtime: a thread pool of its own per search, an atomic
//! stop flag the workers poll, and the driver that polls the caller's
//! stop condition on the calling thread while the pool searches. The
//! engines own the rest (shared memo, cubes, per-worker state); nothing
//! here is global.

use crate::Error;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

/// How long the driver waits for the result before it polls the caller's
/// stop condition again.
const POLL: Duration = Duration::from_millis(1);

/// The pool a parallel search runs on, built for one search and dropped
/// with it: `threads` workers whose stacks fit the recursion limit.
pub(crate) struct Runtime {
    /// The pool.
    pub(crate) pool: ThreadPool,
    /// How many workers it has.
    threads: usize,
}

impl Runtime {
    /// Starts a pool of `threads` workers with stacks of `stack_size`
    /// bytes, or says why it cannot ([`Error::ThreadPool`]).
    pub(crate) fn new(threads: usize, stack_size: usize) -> Result<Self, Error> {
        let threads = threads.max(1);
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .stack_size(stack_size)
            .thread_name(|i| format!("linlog-search-{i}"))
            .build()
            .map_err(|e| Error::ThreadPool(threads, e.to_string()))?;
        Ok(Self { pool, threads })
    }

    /// Returns how many workers the pool has.
    pub(crate) fn threads(&self) -> usize {
        self.threads
    }

    /// Runs `work` on the pool, given the root of the stop flags, and
    /// returns its result. Meanwhile the calling thread polls `stop` once
    /// a millisecond and raises the flag when it fires, so the caller's
    /// condition needs to be neither `Send` nor fast. A panic in `work`
    /// propagates to the caller.
    pub(crate) fn drive<T: Send>(
        &self,
        stop: &mut dyn FnMut() -> bool,
        work: impl FnOnce(Flags<'_>) -> T + Send,
    ) -> T {
        let flag = AtomicBool::new(false);
        let (sender, receiver) = channel();
        let result = self.pool.in_place_scope(|scope| {
            let flag = &flag;
            scope.spawn(move |_| {
                let result = work(Flags::root(flag));
                // The receiver is gone only when the caller's loop ended,
                // which it does only on a result.
                let _ = sender.send(result);
            });
            loop {
                match receiver.recv_timeout(POLL) {
                    Ok(result) => return Some(result),
                    Err(RecvTimeoutError::Timeout) => {
                        if stop() {
                            flag.store(true, Ordering::Relaxed);
                        }
                    }
                    // The task ended without a result: it panicked, and the
                    // scope resumes the panic once the loop lets it end.
                    Err(RecvTimeoutError::Disconnected) => return None,
                }
            }
        });
        result.expect("a task that ends without a result panicked, which the scope propagates")
    }
}

/// The stop flags a worker polls: its own, which a sibling raises to
/// cancel it once their common alternative is settled, and its
/// ancestors', up to the root flag the driver raises for the caller's stop
/// condition. A chain is as long as the nesting of parallel choices, a
/// few links at most.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Flags<'a> {
    /// This level's flag.
    flag: &'a AtomicBool,
    /// The enclosing level's flags.
    parent: Option<&'a Flags<'a>>,
}

impl<'a> Flags<'a> {
    /// The root of a chain: the driver's flag.
    pub(crate) fn root(flag: &'a AtomicBool) -> Self {
        Self { flag, parent: None }
    }

    /// A level below this one, with a flag of its own.
    pub(crate) fn child<'b>(&'b self, flag: &'b AtomicBool) -> Flags<'b>
    where
        'a: 'b,
    {
        Flags {
            flag,
            parent: Some(self),
        }
    }

    /// Whether any flag of the chain is raised.
    pub(crate) fn raised(&self) -> bool {
        let mut level = Some(self);
        while let Some(flags) = level {
            if flags.flag.load(Ordering::Relaxed) {
                return true;
            }
            level = flags.parent;
        }
        false
    }

    /// Whether this level's own flag is raised, whatever the ancestors'.
    pub(crate) fn own(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}
