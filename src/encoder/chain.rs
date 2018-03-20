use std::marker::PhantomData;

use async_codec::{AsyncEncode, AsyncEncodeLen, PollEnc};
use async_codec::PollEnc::{Done, Progress, Pending, Errored};
use futures_core::task::Context;
use futures_io::AsyncWrite;

enum State<S, T> {
    First(S, T),
    Second(T),
}

/// Wraps two `AsyncEncode`s and encodes them in sequence.
pub struct Chain<W, S, T>(State<S, T>, PhantomData<W>);

impl<W, S, T> Chain<W, S, T> {
    /// Create new `Chain` which first encodes the given `S` and then encodes the given `T`.
    pub fn new(first: S, second: T) -> Chain<W, S, T> {
        Chain(State::First(first, second), PhantomData)
    }
}

impl<W, S, T> AsyncEncode<W> for Chain<W, S, T>
    where W: AsyncWrite,
          S: AsyncEncode<W>,
          T: AsyncEncode<W>
{
    fn poll_encode(mut self, cx: &mut Context, writer: &mut W) -> PollEnc<Self> {
        match self.0 {
            State::First(first, second) => {
                match first.poll_encode(cx, writer) {
                    Done(written) => {
                        self.0 = State::Second(second);
                        Progress(self, written)
                    }
                    Progress(first, written) => {
                        self.0 = State::First(first, second);
                        Progress(self, written)
                    }
                    Pending(first) => {
                        self.0 = State::First(first, second);
                        Pending(self)
                    }
                    Errored(err) => Errored(err),
                }
            }

            State::Second(second) => {
                match second.poll_encode(cx, writer) {
                    Done(written) => Done(written),
                    Progress(second, written) => {
                        self.0 = State::Second(second);
                        Progress(self, written)
                    }
                    Pending(second) => {
                        self.0 = State::Second(second);
                        Pending(self)
                    }
                    Errored(err) => Errored(err),
                }
            }
        }
    }
}

impl<W, S, T> AsyncEncodeLen<W> for Chain<W, S, T>
    where W: AsyncWrite,
          S: AsyncEncodeLen<W>,
          T: AsyncEncodeLen<W>
{
    fn remaining_bytes(&self) -> usize {
        match self.0 {
            State::First(ref first, ref second) => {
                first.remaining_bytes() + second.remaining_bytes()
            }
            State::Second(ref second) => second.remaining_bytes(),
        }
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
