//! AIWEB controlled-fixture POC, NOT a browser engine and NOT a security sandbox.
//! Trust assumptions are deliberately narrow: no internet, no untrusted JS, no cross-process IPC.
pub mod action;
pub mod capabilities;
pub mod effects;
pub mod events;
pub mod graph;
pub mod scheduler;
pub mod security;
pub mod state;
pub mod time;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidDag, ResourceExhausted, Unauthorized, Conflict, StaleSnapshot,
    InvalidPrecondition, InvalidLocator, ActionabilityFailed, UnsupportedFeature,
    IdempotencyKeyReuse, StaleLease, EffectUncertain, EventGap, Expired,
}
pub type EngineResult<T> = Result<T, EngineError>;
