use async_codec::{AsyncDecode, PollDec};
use async_codec::PollDec::{Done, Progress, Pending, Errored};
use futures_core::task::Context;
use futures_io::AsyncRead;

enum State<S, T, F> {
    First(S, F),
    Second(T),
}

/// Run a decoder and use the produced item to construct the next decoder to run.
pub struct AndThen<S, T, F>(State<S, T, F>);

impl<S, T, F> AndThen<S, T, F> {
    /// Run a decoder and use the produced item to construct the next decoder to run.
    pub fn new(first: S, f: F) -> AndThen<S, T, F> {
        AndThen(State::First(first, f))
    }
}

impl<S, T, F> AsyncDecode for AndThen<S, T, F>
    where S: AsyncDecode,
          T: AsyncDecode<Error = S::Error>,
          F: FnOnce(S::Item) -> T
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll_decode<R: AsyncRead>(mut self,
                                 cx: &mut Context,
                                 reader: &mut R)
                                 -> PollDec<Self::Item, Self, Self::Error> {
        match self.0 {
            State::First(first, f) => {
                match first.poll_decode(cx, reader) {
                    Done(item, read) => {
                        self.0 = State::Second(f(item));
                        Progress(self, read)
                    }
                    Progress(first, read) => {
                        self.0 = State::First(first, f);
                        Progress(self, read)
                    }
                    Pending(first) => {
                        self.0 = State::First(first, f);
                        Pending(self)
                    }
                    Errored(err) => Errored(err),
                }
            }
            State::Second(second) => {
                match second.poll_decode(cx, reader) {
                    Done(item, read) => Done(item, read),
                    Progress(second, read) => {
                        self.0 = State::Second(second);
                        Progress(self, read)
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
