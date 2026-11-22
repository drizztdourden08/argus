//! Adapters between the ports of `argus-overlord` and the server's own world (I-159): the key
//! store, the real child host and the transcript files.

pub mod approvals;
pub mod feed;
pub mod gate;
pub mod host;
pub mod key;
pub mod rules;
pub mod spend;
pub mod transcript;
