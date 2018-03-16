use std::marker::PhantomData;

use async_codec::{AsyncDecode, DecodeError};
use futures_core::Poll;
use futures_core::Async::Ready;
use futures_core::task::Context;
use futures_io::AsyncRead;

/// Change the return type of a decoder by mapping its item through a function.
pub struct Map<R, D, F> {
    decoder: D,
    f: Option<F>,
    _r: PhantomData<R>,
}

impl<R, D, F> Map<R, D, F> {
    /// Chain a compution on the result of a decoder.
    pub fn new(decoder: D, f: F) -> Map<R, D, F> {
        Map {
            decoder,
            f: Some(f),
            _r: PhantomData,
        }
    }
}

impl<R, D, F, U> AsyncDecode<R> for Map<R, D, F>
    where R: AsyncRead,
          D: AsyncDecode<R>,
          F: FnOnce(D::Item) -> U
{
    type Item = U;
    type Error = D::Error;

    fn poll_decode(&mut self,
                   cx: &mut Context,
                   reader: &mut R)
                   -> Poll<(Option<Self::Item>, usize), DecodeError<Self::Error>> {
        match try_ready!(self.decoder.poll_decode(cx, reader)) {
            (None, read) => Ok(Ready((None, read))),
            (Some(item), read) => {
                Ok(Ready((Some(item).map(self.f
                                             .take()
                                             .expect("polled map decoder after completion")),
                          read)))
            }
        }
    }
}
