use async_codec::{AsyncDecode, PollDec};
use async_codec::PollDec::{Done, Progress, Pending, Errored};
use futures_core::task::Context;
use futures_io::AsyncRead;

enum State<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>
{
    First(S, T),
    Second(T, S::Item),
}

/// Chain two decoders, running them in sequence.
pub struct Chain<R, S, T>(State<R, S, T>)
    where R: AsyncRead,
          S: AsyncDecode<R>;

impl<R, S, T> Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>
{
    /// Create new `Chain` which first decodes via the given `S` and then decodes via the given `T`.
    pub fn new(first: S, second: T) -> Chain<R, S, T> {
        Chain(State::First(first, second))
    }
}

impl<R, S, T> AsyncDecode<R> for Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>,
          T: AsyncDecode<R, Error = S::Error>
{
    type Item = (S::Item, T::Item);
    type Error = S::Error;

    fn poll_decode(mut self,
                   cx: &mut Context,
                   reader: &mut R)
                   -> PollDec<Self::Item, Self, Self::Error> {
        match self.0 {
            State::First(first, second) => {
                match first.poll_decode(cx, reader) {
                    Done(item, read) => {
                        self.0 = State::Second(second, item);
                        Progress(self, read)
                    }
                    Progress(first, read) => {
                        self.0 = State::First(first, second);
                        Progress(self, read)
                    }
                    Pending(first) => {
                        self.0 = State::First(first, second);
                        Pending(self)
                    }
                    Errored(err) => Errored(err),
                }
            }

            State::Second(second, first_item) => {
                match second.poll_decode(cx, reader) {
                    Done(item, read) => Done((first_item, item), read),
                    Progress(second, read) => {
                        self.0 = State::Second(second, first_item);
                        Progress(self, read)
                    }
                    Pending(second) => {
                        self.0 = State::Second(second, first_item);
                        Pending(self)
                    }
                    Errored(err) => Errored(err),
                }
            }
        }
    }
}
