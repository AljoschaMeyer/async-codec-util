//! Utilities for working with the traits from the
//! [async-codec](https://crates.io/crates/async-codec) crate.
#![deny(missing_docs)]

extern crate async_codec;
extern crate futures_core;
extern crate futures_io;

#[cfg(test)]
extern crate async_ringbuffer;
#[cfg(test)]
extern crate atm_io_utils;
#[macro_use]
#[cfg(test)]
extern crate quickcheck;

mod decoder;
pub use decoder::Decoder;
mod encoder;
pub use encoder::Encoder;

/// Decode a value from an `AsyncRead`, using an `AsyncDecode`.
pub fn decode<R, D>(reader: R, dec: D) -> decoder::Decoder<R, D> {
    decoder::Decoder::new(reader, dec)
}

/// Encode a value into an `AsyncWrite`, using an `AsyncEncode`.
pub fn encode<W, C>(writer: W, co: C) -> encoder::Encoder<W, C> {
    encoder::Encoder::new(writer, co)
}
