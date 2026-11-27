//! The handover before an account switch: the `SW-n` cycle and its `H-n` handovers, one per
//! lead that was working (inside a turn or with a live agent record) when the cycle started.
//!
//! Estimate ([`estimator`] works out the ask point), ask point, handover request, then all ready or the switch point: [`protocol`] is
//! that cycle as a pure state machine, [`desk`] carries its effects out over the
//! [`Threads`](crate::ports::Threads) and [`HandoverStore`](crate::ports::HandoverStore) ports,
//! [`texts`] holds what the leads read. The countdown and the switch belong to the rules, the
//! resume to the resumer; both read the cycle and call the desk.

pub mod desk;
pub mod estimator;
pub mod protocol;
pub mod records;
pub mod texts;

pub use desk::{Desk, DeskError, LeadAction, LeadReply, Outcome, StartRequest};
pub use estimator::{AgentLoad, CycleCost, LeadEstimate, LeadInput};
pub use protocol::{CycleEvent, Effect, ProtocolError, READY_REASON, WakeKind};
pub use records::{
    AgentStop, Cycle, CycleMode, CyclePhase, Estimate, Handover, HandoverState, Lead, Via,
};
