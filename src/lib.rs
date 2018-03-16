//! Utilities for working with the traits from the
//! [async-codec](https://crates.io/crates/async-codec) crate.
#![deny(missing_docs)]

extern crate async_codec;
#[macro_use(try_ready)]
extern crate futures_core;
extern crate futures_io;
extern crate futures_executor;
extern crate futures_util;

pub mod encoder;
pub mod decoder;
pub mod testing;

use async_codec::{AsyncEncode, AsyncEncodeLen, AsyncDecode, DecodeError};
use futures_core::{Future, Poll};
use futures_core::Async::{Ready, Pending};
use futures_core::task::Context;
use futures_io::{AsyncRead, AsyncWrite, Error as FutIoErr};

/// Encode a value into an `AsyncWrite`, using an `AsyncEncode`.
pub fn encode<W, C>(writer: W, co: C) -> Encoder<W, C> {
    Encoder::new(writer, co)
}

/// Future for fully encoding an `AsyncEncode` into an `AsyncWrite`.
pub struct Encoder<W, C> {
    writer: Option<W>,
    co: C,
    written: usize,
}

impl<W, C> Encoder<W, C> {
    /// Create a new `Encoder` wrapping an `AsyncWrite` and consuming an `AsyncEncode`.
    pub fn new(writer: W, co: C) -> Encoder<W, C> {
        Encoder {
            writer: Some(writer),
            co,
            written: 0,
        }
    }
}

impl<W, C> Encoder<W, C>
    where W: AsyncWrite,
          C: AsyncEncodeLen<W>
{
    /// Return the exact number of bytes this will still write.
    pub fn remaining_bytes(&self) -> usize {
        self.co.remaining_bytes()
    }
}

impl<W, C> Future for Encoder<W, C>
    where W: AsyncWrite,
          C: AsyncEncode<W>
{
    type Item = (W, usize);
    type Error = (W, FutIoErr);

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let mut writer = self.writer
            .take()
            .expect("Polled future after completion");
        match self.co.poll_encode(cx, &mut writer) {
            Ok(Ready(0)) => Ok(Ready((writer, self.written))),
            Ok(Ready(written)) => {
                self.written += written;
                self.writer = Some(writer);
                self.poll(cx)
            }
            Ok(Pending) => {
                self.writer = Some(writer);
                Ok(Pending)
            }
            Err(err) => Err((writer, err)),
        }
    }
}

/// Decode a value from an `AsyncRead`, using an `AsyncDecode`.
pub fn decode<R, D>(reader: R, dec: D) -> Decoder<R, D> {
    Decoder::new(reader, dec)
}

/// Future for fully decoding an `AsyncDecode` from an `AsyncRead`.
pub struct Decoder<R, D> {
    reader: Option<R>,
    dec: D,
    read: usize,
}

impl<R, D> Decoder<R, D> {
    /// Create a new `Decoder` wrapping an `AsyncRead` and consuming an `AsyncDecode`.
    pub fn new(reader: R, dec: D) -> Decoder<R, D> {
        Decoder {
            reader: Some(reader),
            dec,
            read: 0,
        }
    }
}

impl<R, D> Future for Decoder<R, D>
    where R: AsyncRead,
          D: AsyncDecode<R>
{
    type Item = (R, D::Item, usize);
    type Error = (R, DecodeError<D::Error>);

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let mut reader = self.reader
            .take()
            .expect("Polled future after completion");
        match self.dec.poll_decode(cx, &mut reader) {
            Ok(Ready((Some(decoded), read))) => Ok(Ready((reader, decoded, self.read + read))),
            Ok(Ready((None, read))) => {
                self.read += read;
                self.reader = Some(reader);
                self.poll(cx)
            }
            Ok(Pending) => {
                self.reader = Some(reader);
                Ok(Pending)
            }
            Err(err) => Err((reader, err)),
        }
    }
}
