//! The domain — the heart of the gossip ring.
//!
//! Nothing in here knows about OpenAI, rodio, the filesystem or clap. It only
//! describes *what* TxtTattler does (read text, turn it into speech, play it)
//! and leaves the dirty *how* to the infrastructure adapters.

pub mod entities;
pub mod ports;
