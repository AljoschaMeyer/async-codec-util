//! Utilities for working with the traits from the
//! [async-codec](https://crates.io/crates/async-codec) crate.
#![deny(missing_docs)]

extern crate async_codec;
extern crate futures_core;
extern crate futures_io;
extern crate futures_executor;
extern crate futures_util;

#[cfg(test)]
extern crate async_byteorder;
#[cfg(test)]
extern crate async_ringbuffer;
#[cfg(test)]
extern crate atm_io_utils;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck;

pub mod encoder;
pub mod decoder;
pub mod testing;

use async_codec::{AsyncEncode, AsyncEncodeLen, AsyncDecode, DecodeError, PollEnc, PollDec};
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
    enc: Option<C>,
    written: usize,
}

impl<W, C> Encoder<W, C> {
    /// Create a new `Encoder` wrapping an `AsyncWrite` and consuming an `AsyncEncode`.
    pub fn new(writer: W, enc: C) -> Encoder<W, C> {
        Encoder {
            writer: Some(writer),
            enc: Some(enc),
            written: 0,
        }
    }
}

impl<W, C> Encoder<W, C>
    where W: AsyncWrite,
          C: AsyncEncodeLen<W>
{
    /// Return the exact number of bytes this will still write.
    ///
    /// Panics if called after the future completed.
    pub fn remaining_bytes(&mut self) -> usize {
        let enc = self.enc
            .take()
            .expect("Used encoder future after completion");
        let remaining = enc.remaining_bytes();
        self.enc = Some(enc);
        remaining
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
            .expect("Polled encoder future after completion");
        let enc = self.enc
            .take()
            .expect("Polled encoder future after completion");

        match enc.poll_encode(cx, &mut writer) {
            PollEnc::Done(written) => Ok(Ready((writer, self.written + written))),
            PollEnc::Progress(enc, written) => {
                self.written += written;
                self.writer = Some(writer);
                self.enc = Some(enc);
                self.poll(cx)
            }
            PollEnc::Pending(enc) => {
                self.writer = Some(writer);
                self.enc = Some(enc);
                Ok(Pending)
            }
            PollEnc::Errored(err) => Err((writer, err)),
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
    dec: Option<D>,
    read: usize,
}

impl<R, D> Decoder<R, D> {
    /// Create a new `Decoder` wrapping an `AsyncRead` and consuming an `AsyncDecode`.
    pub fn new(reader: R, dec: D) -> Decoder<R, D> {
        Decoder {
            reader: Some(reader),
            dec: Some(dec),
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
            .expect("Polled decoder future after completion");
        let dec = self.dec
            .take()
            .expect("Polled decoder future after completion");

        match dec.poll_decode(cx, &mut reader) {
            PollDec::Done(item, read) => Ok(Ready((reader, item, self.read + read))),
            PollDec::Progress(dec, read) => {
                self.read += read;
                self.reader = Some(reader);
                self.dec = Some(dec);
                self.poll(cx)
            }
            PollDec::Pending(dec) => {
                self.reader = Some(reader);
                self.dec = Some(dec);
                Ok(Pending)
            }
            PollDec::Errored(err) => Err((reader, err)),
        }
    }
}
