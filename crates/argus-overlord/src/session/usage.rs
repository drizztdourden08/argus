//! The usage of the hosted child's session, read from its lines: the tokens and cost of every
//! `result`, the context of the last assistant message and the model of the last `init`. It lives
//! in memory and starts again at a new session id.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use crate::protocol::{Event, Init, Message, Outcome, Usage};
use crate::spend::{Last, delta};

/// The context window of a model, in tokens.
const WINDOW: u64 = 200_000;
/// The context window of a `[1m]` model.
const WINDOW_1M: u64 = 1_000_000;

/// The counters of one `result`, or their sum over a session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    /// The cost of the turn: the rise of the process's cumulative cost.
    pub cost_usd: f64,
    /// `num_turns` of the result.
    pub turns: u64,
    pub duration_ms: u64,
}

impl Totals {
    fn of(outcome: &Outcome, cost_usd: f64) -> Self {
        let usage = outcome.usage.clone().unwrap_or_default();
        Self {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_read_tokens: usage.cache_read_input_tokens,
            cache_write_tokens: usage.cache_creation_input_tokens,
            cost_usd,
            turns: outcome.num_turns.unwrap_or(0),
            duration_ms: outcome.duration_ms.unwrap_or(0),
        }
    }

    fn add(&mut self, turn: &Totals) {
        self.input_tokens += turn.input_tokens;
        self.output_tokens += turn.output_tokens;
        self.cache_read_tokens += turn.cache_read_tokens;
        self.cache_write_tokens += turn.cache_write_tokens;
        self.cost_usd += turn.cost_usd;
        self.turns += turn.turns;
        self.duration_ms += turn.duration_ms;
    }
}

/// The tokens a request sent: its input, read from the cache and written to it.
fn context_of(usage: &Usage) -> u64 {
    usage.input_tokens + usage.cache_read_input_tokens + usage.cache_creation_input_tokens
}

/// The context window of `model`: 1,000,000 tokens for an id ending in `[1m]`, else 200,000.
#[must_use]
pub fn window_tokens(model: &str) -> u64 {
    if model.ends_with("[1m]") {
        WINDOW_1M
    } else {
        WINDOW
    }
}

/// What the session used so far.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tally {
    pub session_id: Option<String>,
    /// The sum over every `result` of the session.
    pub session: Totals,
    /// The last `result`; none before the first.
    pub last_turn: Option<Totals>,
    /// Input, cache read and cache write tokens of the last assistant message of the main thread.
    pub context_tokens: u64,
    /// The model of the last `init`.
    pub model: Option<String>,
    /// The last cumulative cost, to price the next turn.
    last: Option<Last>,
    /// Counts the child processes, so a new one prices from zero.
    process: u64,
}

impl Tally {
    /// A new child process: its cumulative cost starts from zero.
    pub fn spawned(&mut self) {
        self.process += 1;
    }

    /// Reads one decoded line of the child.
    pub fn see(&mut self, event: &Event) {
        match event {
            Event::Init(init) => self.init(init),
            Event::Assistant(message) => self.assistant(message),
            Event::Result(outcome) => self.result(outcome),
            _ => {}
        }
    }

    fn init(&mut self, init: &Init) {
        if self.session_id.as_deref() != Some(init.session_id.as_str()) {
            *self = Tally {
                process: self.process,
                session_id: Some(init.session_id.clone()),
                ..Tally::default()
            };
        }
        if init.model.is_some() {
            self.model.clone_from(&init.model);
        }
    }

    fn assistant(&mut self, message: &Message) {
        if message.parent_tool_use_id.is_some() {
            return;
        }
        if let Some(usage) = &message.usage {
            self.context_tokens = context_of(usage);
        }
    }

    fn result(&mut self, outcome: &Outcome) {
        let cost_usd = match outcome.total_cost_usd {
            Some(cumulative_usd) => {
                let spent = delta(self.last, self.process, cumulative_usd);
                self.last = Some(Last {
                    process_seq: self.process,
                    cumulative_usd,
                });
                spent
            }
            None => 0.0,
        };
        let turn = Totals::of(outcome, cost_usd);
        self.session.add(&turn);
        self.last_turn = Some(turn);
    }
}

/// The tally, shared: the supervisor adds to it and the server reads it. Cheap to clone; every
/// clone sees the same tally, so one kept by the server outlives a restart of the child.
#[derive(Debug, Clone, Default)]
pub struct Meter {
    tally: Arc<Mutex<Tally>>,
}

impl Meter {
    fn tally(&self) -> MutexGuard<'_, Tally> {
        self.tally.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A new child process was spawned.
    pub fn spawned(&self) {
        self.tally().spawned();
    }

    /// Reads one decoded line of the child.
    pub fn see(&self, event: &Event) {
        self.tally().see(event);
    }

    /// The tally as it is now.
    #[must_use]
    pub fn snapshot(&self) -> Tally {
        self.tally().clone()
    }
}

#[cfg(test)]
mod specs {
    use super::*;
    use crate::protocol::decode_line;

    const INIT: &str = r#"{"type":"system","subtype":"init","session_id":"s1","model":"claude-opus-5-5[1m]"}"#;
    const ASSISTANT: &str = r#"{"type":"assistant","message":{"content":[],"usage":{"input_tokens":4,"cache_read_input_tokens":900,"cache_creation_input_tokens":96,"output_tokens":7}}}"#;
    const SIDE: &str = r#"{"type":"assistant","parent_tool_use_id":"t1","message":{"content":[],"usage":{"input_tokens":5}}}"#;
    const ONE: &str = r#"{"type":"result","total_cost_usd":0.25,"duration_ms":1000,"num_turns":2,"usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":100,"cache_creation_input_tokens":20}}"#;
    const TWO: &str = r#"{"type":"result","total_cost_usd":0.75,"duration_ms":500,"num_turns":1,"usage":{"input_tokens":1,"output_tokens":2,"cache_read_input_tokens":3,"cache_creation_input_tokens":4}}"#;

    fn tally(lines: &[&str]) -> Tally {
        let mut tally = Tally::default();
        tally.spawned();
        for line in lines {
            tally.see(&decode_line(line).expect("line"));
        }
        tally
    }

    // test-tag: critical
    #[test]
    fn the_results_of_a_session_are_summed() {
        let seen = tally(&[INIT, ONE, TWO]);
        assert_eq!(seen.session.input_tokens, 11);
        assert_eq!(seen.session.output_tokens, 7);
        assert_eq!(seen.session.cache_read_tokens, 103);
        assert_eq!(seen.session.cache_write_tokens, 24);
        assert_eq!((seen.session.turns, seen.session.duration_ms), (3, 1500));
        assert!((seen.session.cost_usd - 0.75).abs() < 1e-9);
        let last = seen.last_turn.expect("last turn");
        assert!((last.cost_usd - 0.5).abs() < 1e-9, "cumulative to per turn");
        assert_eq!(last.input_tokens, 1);
    }

    #[test]
    fn the_context_is_the_last_assistant_message_of_the_main_thread() {
        let seen = tally(&[INIT, ASSISTANT]);
        assert_eq!(seen.context_tokens, 1000);
        let other = tally(&[INIT, ASSISTANT, SIDE]);
        assert_eq!(other.context_tokens, 1000);
        assert_eq!(seen.model.as_deref(), Some("claude-opus-5-5[1m]"));
    }

    #[test]
    fn a_new_session_id_starts_again_and_a_new_process_prices_from_zero() {
        let mut seen = tally(&[INIT, ONE]);
        seen.spawned();
        seen.see(&decode_line(INIT).expect("init"));
        seen.see(&decode_line(ONE).expect("result"));
        assert!((seen.session.cost_usd - 0.5).abs() < 1e-9, "same session");
        seen.see(&decode_line(&INIT.replace("s1", "s2")).expect("init"));
        assert_eq!(seen.session, Totals::default());
        assert_eq!(seen.last_turn, None);
    }

    #[test]
    fn the_window_follows_the_model() {
        assert_eq!(window_tokens("claude-opus-5-5[1m]"), 1_000_000);
        assert_eq!(window_tokens("claude-opus-5-5"), 200_000);
    }
}
