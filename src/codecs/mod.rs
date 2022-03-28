//! A collection of codecs that can be used to transform between bytes streams /
//! byte messages, byte frames and structured events.

#![deny(missing_docs)]

mod ready_frames;

pub use ready_frames::ReadyFrames;
