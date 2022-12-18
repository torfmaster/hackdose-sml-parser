//! Crate to read and parse Smart Message Language emitted by smart meters
//!
//! The library consists of three layers:
//!
//! # Transport Layer
//!
//! The [transport] layer consists of primitives to parse an SML message from raw bytes.
//!
//! # Application Layer
//!
//! The application layer handles parsing of SML messages from an SML message body.
//! It reads actual data from SML messages
//!
//! # Message Stream
//! This reflects the main use-case for using this crate: It converts a byte-stream
//! to a stream of valid SML messages.
//!
pub mod application;
pub mod message_stream;
pub mod transport;
