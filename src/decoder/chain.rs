use async_codec::{AsyncDecode, PollDec};
use async_codec::PollDec::{Done, Progress, Pending, Errored};
use futures_core::task::Context;
use futures_io::AsyncRead;

enum State<S, T>
    where S: AsyncDecode
{
    First(S, T),
    Second(T, S::Item),
}

/// Chain two decoders, running them in sequence.
pub struct Chain<S, T>(State<S, T>) where S: AsyncDecode;

impl<S, T> Chain<S, T>
    where S: AsyncDecode
{
    /// Create new `Chain` which first decodes via the given `S` and then decodes via the given `T`.
    pub fn new(first: S, second: T) -> Chain<S, T> {
        Chain(State::First(first, second))
    }
}

impl<S, T> AsyncDecode for Chain<S, T>
    where S: AsyncDecode,
          T: AsyncDecode<Error = S::Error>
{
    type Item = (S::Item, T::Item);
    type Error = S::Error;

    fn poll_decode<R: AsyncRead>(mut self,
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
