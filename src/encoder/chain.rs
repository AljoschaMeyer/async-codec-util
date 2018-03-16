use std::marker::PhantomData;

use async_codec::{AsyncEncode, AsyncEncodeLen};
use futures_core::Poll;
use futures_core::Async::Ready;
use futures_core::task::Context;
use futures_io::{AsyncWrite, Error as FutIoErr};

/// Wraps two `AsyncEncode`s and encodes them in sequence.
pub struct Chain<W, S, T> {
    first: S,
    second: T,
    encode_first: bool,
    _w: PhantomData<W>,
}

impl<W, S, T> Chain<W, S, T> {
    /// Create new `Chain` which first encodes the given `S` and then encodes the given `T`.
    pub fn new(first: S, second: T) -> Chain<W, S, T> {
        Chain {
            first,
            second,
            encode_first: true,
            _w: PhantomData,
        }
    }
}

impl<W, S, T> AsyncEncode<W> for Chain<W, S, T>
    where W: AsyncWrite,
          S: AsyncEncode<W>,
          T: AsyncEncode<W>
{
    fn poll_encode(&mut self, cx: &mut Context, writer: &mut W) -> Poll<usize, FutIoErr> {
        if self.encode_first {
            let written = try_ready!(self.first.poll_encode(cx, writer));
            if written == 0 {
                self.encode_first = false;
                self.poll_encode(cx, writer)
            } else {
                Ok(Ready(written))
            }
        } else {
            self.second.poll_encode(cx, writer)
        }
    }
}

impl<W, S, T> AsyncEncodeLen<W> for Chain<W, S, T>
    where W: AsyncWrite,
          S: AsyncEncodeLen<W>,
          T: AsyncEncodeLen<W>
{
    fn remaining_bytes(&self) -> usize {
        self.first.remaining_bytes() + self.second.remaining_bytes()
    }
}

// TODO test via test_codec_len with the decoder Chain
