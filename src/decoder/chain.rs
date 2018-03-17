use std::marker::PhantomData;

use async_codec::{AsyncDecode, DecodeError};
use futures_core::Poll;
use futures_core::Async::Ready;
use futures_core::task::Context;
use futures_io::AsyncRead;

/// Chain two decoders, running them in sequence.
pub struct Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>
{
    first: S,
    second: T,
    first_item: Option<S::Item>,
    _r: PhantomData<R>,
}

impl<R, S, T> Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>
{
    /// Create new `Chain` which first decodes via the given `S` and then decodes via the given `T`.
    pub fn new(first: S, second: T) -> Chain<R, S, T> {
        Chain {
            first,
            second,
            first_item: None,
            _r: PhantomData,
        }
    }
}

impl<R, S, T> AsyncDecode<R> for Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>,
          T: AsyncDecode<R, Error = S::Error>
{
    type Item = (S::Item, T::Item);
    type Error = S::Error;

    fn poll_decode(&mut self,
                   cx: &mut Context,
                   reader: &mut R)
                   -> Poll<(Option<Self::Item>, usize), DecodeError<Self::Error>> {
        if self.first_item.is_none() {
            match try_ready!(self.first.poll_decode(cx, reader)) {
                (None, read) => Ok(Ready((None, read))),
                (Some(item), read) => {
                    self.first_item = Some(item);
                    Ok(Ready((None, read)))
                }
            }
        } else {
            match try_ready!(self.second.poll_decode(cx, reader)) {
                (None, read) => Ok(Ready((None, read))),
                (Some(item), read) => {
                    Ok(Ready((Some((self.first_item.take().unwrap(), item)), read)))
                }
            }
        }
    }
}
