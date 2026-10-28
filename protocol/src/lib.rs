#[macro_use]
mod macros;

pub mod event;
pub mod frame;
pub mod geckoview;
pub mod inbound;
pub mod message;
pub mod rikka;

pub use event::{Event, ForEmbedder, ForGecko, Request};
pub use frame::{Decoder, FrameError, MAX_FRAME, read_frame, write_frame};
pub use message::{
    CallbackId, EmbedderMessage, EventMessage, GeckoMessage, PROTOCOL, Reply, Target, WindowId,
};
