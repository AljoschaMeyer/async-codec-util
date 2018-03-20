use async_codec::{AsyncDecode, PollDec};
use async_codec::PollDec::{Done, Progress, Pending, Errored};
use futures_core::task::Context;
use futures_io::AsyncRead;

/// Change the return type of a decoder by mapping its item through a function.
pub struct Map<D, F> {
    dec: D,
    f: F,
}

impl<D, F> Map<D, F> {
    /// Chain a compution on the result of a decoder.
    pub fn new(dec: D, f: F) -> Map<D, F> {
        Map { dec, f }
    }
}

impl<D, F, U> AsyncDecode for Map<D, F>
    where D: AsyncDecode,
          F: FnOnce(D::Item) -> U
{
    type Item = U;
    type Error = D::Error;

    fn poll_decode<R: AsyncRead>(mut self,
                                 cx: &mut Context,
                                 reader: &mut R)
                                 -> PollDec<Self::Item, Self, Self::Error> {
        match self.dec.poll_decode(cx, reader) {
            Done(item, read) => Done((self.f)(item), read),
            Progress(dec, read) => {
                self.dec = dec;
                Progress(self, read)
            }
            Pending(dec) => {
                self.dec = dec;
                Pending(self)
            }
            Errored(err) => Errored(err),
        }
    }
}
