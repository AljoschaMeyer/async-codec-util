use async_codec::{AsyncEncode, AsyncEncodeLen};
use futures_core::{Future, Poll};
use futures_core::Async::{Ready, Pending};
use futures_core::task::Context;
use futures_io::{AsyncWrite, Error as FutIoErr};

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
