//! Shared query execution policy for the HTTP and MCP interfaces.
//!
//! A [`QueryPolicy`] turns the optional `--query-timeout` setting into a
//! per-query [`QueryExecution`] that carries an Oxigraph cancellation token and
//! an absolute deadline. A single background thread cancels tokens whose
//! deadline has passed, so both interfaces share one timeout implementation.

use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use oxigraph::sparql::CancellationToken;

/// Parses the `--query-timeout` value as a positive whole number of seconds.
///
/// Used as the clap `value_parser` for both interfaces.
pub fn parse_query_timeout(value: &str) -> Result<Duration, String> {
    let seconds: u64 = value.parse().map_err(|_| {
        format!("query timeout must be a whole number of seconds, but was `{value}`")
    })?;
    if seconds == 0 {
        return Err("query timeout must be at least 1 second".to_owned());
    }
    let timeout = Duration::from_secs(seconds);
    if Instant::now().checked_add(timeout).is_none() {
        return Err(format!("query timeout is too large: {seconds} seconds"));
    }
    Ok(timeout)
}

/// Why a query must stop before producing a successful result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Interruption {
    /// The absolute deadline passed before the query finished.
    TimedOut(Duration),
    /// The client cancelled the request before the deadline.
    Cancelled,
}

/// A cloneable timeout policy shared by every query of one interface.
#[derive(Clone)]
pub(crate) struct QueryPolicy {
    /// Maximum elapsed time for one query, or unlimited execution.
    timeout: Option<Duration>,
    /// Registry responsible for cancelling query tokens at their deadlines.
    deadlines: QueryDeadlines,
}

impl QueryPolicy {
    /// Builds a policy from the optional timeout. `None` leaves execution
    /// unlimited.
    pub(crate) fn new(timeout: Option<Duration>) -> Self {
        Self {
            timeout,
            deadlines: QueryDeadlines::new(),
        }
    }

    /// Starts tracking one query, returning its execution handle.
    pub(crate) fn begin(&self) -> QueryExecution {
        let token = CancellationToken::new();
        let budget = self.timeout.and_then(|timeout| {
            Instant::now()
                .checked_add(timeout)
                .map(|deadline| Budget { deadline, timeout })
        });
        let registration = budget
            .as_ref()
            .map(|budget| self.deadlines.register(budget.deadline, token.clone()));
        QueryExecution {
            token,
            budget,
            _registration: registration,
        }
    }
}

#[derive(Clone, Copy)]
/// Absolute expiry and original timeout used to classify and report interruptions.
struct Budget {
    /// Instant at which the query must stop.
    deadline: Instant,
    /// Original duration used in timeout diagnostics.
    timeout: Duration,
}

/// Tracks one running query's deadline and cancellation token.
pub(crate) struct QueryExecution {
    /// Engine token cancelled by a deadline or client request.
    token: CancellationToken,
    /// Deadline and diagnostic duration when a timeout is configured.
    budget: Option<Budget>,
    /// Guard removing this query from the deadline registry when dropped.
    _registration: Option<Registration>,
}

impl QueryExecution {
    /// The cancellation token to hand to the SPARQL evaluator.
    pub(crate) fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Builds an execution whose deadline has already passed, for tests that
    /// exercise the final deadline check without racing a timer.
    #[cfg(test)]
    pub(crate) fn expired(timeout: Duration) -> Self {
        Self {
            token: CancellationToken::new(),
            budget: Some(Budget {
                deadline: Instant::now() - timeout,
                timeout,
            }),
            _registration: None,
        }
    }

    /// Returns the reason the query must stop, or `None` while it may continue.
    ///
    /// A passed deadline reports [`Interruption::TimedOut`] even when the timer
    /// has also cancelled the token, keeping deadline expiry distinct from
    /// client cancellation.
    pub(crate) fn check(&self) -> Option<Interruption> {
        if let Some(budget) = self.budget
            && Instant::now() >= budget.deadline
        {
            return Some(Interruption::TimedOut(budget.timeout));
        }
        if self.token.is_cancelled() {
            return Some(Interruption::Cancelled);
        }
        None
    }
}

/// A lazily started registry that cancels query tokens at their deadlines.
#[derive(Clone)]
struct QueryDeadlines {
    /// Registry state and wakeup signal shared with the timer thread.
    shared: Arc<Shared>,
}

/// Synchronization shared by deadline registration and the timer worker.
struct Shared {
    /// Registered deadlines and timer-thread lifecycle state.
    state: Mutex<State>,
    /// Signal waking the timer when the registry changes.
    signal: Condvar,
}

/// Mutable deadline entries and registration identity allocation.
struct State {
    /// Cancellation tokens ordered by deadline and unique registration ID.
    entries: BTreeMap<(Instant, u64), CancellationToken>,
    /// Identity assigned to the next query registration.
    next_id: u64,
    /// Whether the timer worker has been started.
    thread_started: bool,
}

impl QueryDeadlines {
    /// Create an empty registry whose timer starts on the first registration.
    fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    entries: BTreeMap::new(),
                    next_id: 0,
                    thread_started: false,
                }),
                signal: Condvar::new(),
            }),
        }
    }

    /// Registers a token to cancel at `deadline`, starting the timer thread on
    /// first use. The returned [`Registration`] removes the entry when dropped.
    fn register(&self, deadline: Instant, token: CancellationToken) -> Registration {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("query deadline lock poisoned");
        let id = state.next_id;
        state.next_id += 1;
        state.entries.insert((deadline, id), token);
        if !state.thread_started {
            state.thread_started = true;
            let shared = Arc::clone(&self.shared);
            thread::Builder::new()
                .name("sparqld-query-deadlines".into())
                .spawn(move || run_timer(&shared))
                .expect("spawning the query deadline thread");
        }
        self.shared.signal.notify_all();
        Registration {
            shared: Arc::clone(&self.shared),
            key: (deadline, id),
        }
    }

    #[cfg(test)]
    fn pending(&self) -> usize {
        self.shared
            .state
            .lock()
            .expect("query deadline lock poisoned")
            .entries
            .len()
    }
}

/// Wait for the nearest deadline and cancel all tokens whose deadlines have passed.
fn run_timer(shared: &Arc<Shared>) {
    let mut state = shared.state.lock().expect("query deadline lock poisoned");
    loop {
        match state.entries.keys().next().copied() {
            None => {
                state = shared
                    .signal
                    .wait(state)
                    .expect("query deadline lock poisoned");
            }
            Some((deadline, _)) => {
                let now = Instant::now();
                if deadline <= now {
                    let expired: Vec<(Instant, u64)> = state
                        .entries
                        .range(..=(now, u64::MAX))
                        .map(|(key, _)| *key)
                        .collect();
                    for key in expired {
                        if let Some(token) = state.entries.remove(&key) {
                            token.cancel();
                        }
                    }
                } else {
                    let (next, _) = shared
                        .signal
                        .wait_timeout(state, deadline - now)
                        .expect("query deadline lock poisoned");
                    state = next;
                }
            }
        }
    }
}

/// Removes one query's deadline entry when the query stops.
struct Registration {
    /// Registry from which the entry is removed when this guard is dropped.
    shared: Arc<Shared>,
    /// Deadline and unique ID identifying this query's registry entry.
    key: (Instant, u64),
}

impl Drop for Registration {
    fn drop(&mut self) {
        if let Ok(mut state) = self.shared.state.lock() {
            state.entries.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_whole_number_of_seconds() {
        assert_eq!(parse_query_timeout("20"), Ok(Duration::from_secs(20)));
    }

    #[test]
    fn rejects_non_positive_and_malformed_timeouts() {
        assert!(parse_query_timeout("0").is_err());
        assert!(parse_query_timeout("-1").is_err());
        assert!(parse_query_timeout("abc").is_err());
        assert!(parse_query_timeout("1.5").is_err());
        assert!(parse_query_timeout("20s").is_err());
    }

    #[test]
    fn rejects_an_overflowing_timeout() {
        assert!(parse_query_timeout(&u64::MAX.to_string()).is_err());
    }

    #[test]
    fn cancels_the_token_at_the_deadline() {
        let policy = QueryPolicy::new(Some(Duration::from_millis(50)));
        let execution = policy.begin();

        wait_until(|| execution.token().is_cancelled(), Duration::from_secs(2));

        assert_eq!(
            execution.check(),
            Some(Interruption::TimedOut(Duration::from_millis(50)))
        );
    }

    #[test]
    fn dropping_a_registration_removes_its_entry() {
        let policy = QueryPolicy::new(Some(Duration::from_millis(50)));
        let execution = policy.begin();
        assert_eq!(policy.deadlines.pending(), 1);

        drop(execution);

        assert_eq!(policy.deadlines.pending(), 0);
    }

    #[test]
    fn runs_concurrent_deadlines_independently() {
        let deadlines = QueryDeadlines::new();
        let soon = CancellationToken::new();
        let later = CancellationToken::new();
        let _soon = deadlines.register(Instant::now() + Duration::from_millis(50), soon.clone());
        let _later = deadlines.register(Instant::now() + Duration::from_secs(30), later.clone());

        wait_until(|| soon.is_cancelled(), Duration::from_secs(2));

        assert!(!later.is_cancelled());
    }

    #[test]
    fn reports_a_timeout_even_when_the_timer_fires_late() {
        let execution = QueryExecution {
            token: CancellationToken::new(),
            budget: Some(Budget {
                deadline: Instant::now() - Duration::from_secs(1),
                timeout: Duration::from_secs(5),
            }),
            _registration: None,
        };

        assert_eq!(
            execution.check(),
            Some(Interruption::TimedOut(Duration::from_secs(5)))
        );
    }

    #[test]
    fn reports_cancellation_before_the_deadline() {
        let token = CancellationToken::new();
        let execution = QueryExecution {
            token: token.clone(),
            budget: Some(Budget {
                deadline: Instant::now() + Duration::from_secs(30),
                timeout: Duration::from_secs(30),
            }),
            _registration: None,
        };
        token.cancel();

        assert_eq!(execution.check(), Some(Interruption::Cancelled));
    }

    #[test]
    fn registers_nothing_without_a_timeout() {
        let policy = QueryPolicy::new(None);
        let execution = policy.begin();

        thread::sleep(Duration::from_millis(50));

        assert_eq!(policy.deadlines.pending(), 0);
        assert!(execution.check().is_none());
        assert!(!execution.token().is_cancelled());
    }

    fn wait_until(mut condition: impl FnMut() -> bool, limit: Duration) {
        let deadline = Instant::now() + limit;
        while !condition() {
            assert!(Instant::now() < deadline, "condition was not met in time");
            thread::sleep(Duration::from_millis(5));
        }
    }
}
