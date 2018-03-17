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

#[cfg(test)]
mod tests {
    use atm_io_utils::partial::*;
    use async_ringbuffer::ring_buffer;

    use async_byteorder::{decode_i32_native, decode_u64_native, encode_i32_native,
                          encode_u64_native};
    use super::super::super::testing::test_codec_len;
    use super::super::super::decoder::chain as dec_chain;
    use super::super::super::encoder::chain as enc_chain;

    quickcheck! {
        fn codec(buf_size: usize, read_ops: Vec<PartialOp>, write_ops: Vec<PartialOp>, int_0: i32, int_1: u64) -> bool {
            let mut read_ops = read_ops;
            let mut write_ops = write_ops;
            let (w, r) = ring_buffer(buf_size + 1);
            let w = PartialWrite::new(w, write_ops.drain(..));
            let r = PartialRead::new(r, read_ops.drain(..));

            let test_outcome = test_codec_len(r, w, dec_chain(decode_i32_native(), decode_u64_native()), enc_chain(encode_i32_native(int_0), encode_u64_native(int_1)));
            test_outcome.1 && (test_outcome.0).0 == int_0 && (test_outcome.0).1 == int_1
        }
    }
}
