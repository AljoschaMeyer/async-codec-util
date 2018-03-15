use async_codec::{AsyncDecode, DecodeError};
use futures_core::{Future, Poll};
use futures_core::Async::{Ready, Pending};
use futures_core::task::Context;
use futures_io::AsyncRead;

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
