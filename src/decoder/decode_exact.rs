use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use async_codec::{AsyncDecode, PollDec, DecodeError};
use async_codec::PollDec::{Done, Progress, Pending, Errored};
use atm_io_utils::limited_reader::LimitedReader;
use futures_core::task::Context;
use futures_io::AsyncRead;

/// The error of a `DecodeExact`.
///
/// The inner decoder is limited in the number of bytes it can read. It errors with `UnexpectedEof`
/// when it tries to read more than the allowed amount of bytes.
#[derive(Debug)]
pub enum DecodeExactError<E, I> {
    /// The inner decoder finished decoding too early, after the contained number of bytes.
    /// This error also contains the decoded item.
    Early(I, usize),
    /// The inner decoder errored.
    Inner(E),
}

impl<E: Display, I> Display for DecodeExactError<E, I> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            DecodeExactError::Early(_, read) => {
                write!(f, "Decoder finished early after reading {} bytes", read)
            }
            DecodeExactError::Inner(ref err) => write!(f, "Decode exact inner error: {}", err),
        }
    }
}

impl<E: Error, I: Debug> Error for DecodeExactError<E, I> {
    fn description(&self) -> &str {
        match *self {
            DecodeExactError::Early(_, _) => "decoder finished early",
            DecodeExactError::Inner(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            DecodeExactError::Early(_, _) => None,
            DecodeExactError::Inner(ref err) => Some(err),
        }
    }
}

impl<E, I> From<E> for DecodeExactError<E, I> {
    fn from(err: E) -> DecodeExactError<E, I> {
        DecodeExactError::Inner(err)
    }
}

/// Wraps a decoder and emits an error if it does not stop encoding at exactly the specified byte
/// count.
pub struct DecodeExact<D> {
    target: usize,
    read: usize,
    dec: D,
}

impl<D> DecodeExact<D> {
    /// Create a new `DecodeExact`, delegating to the given `dec` and checking that it decodes
    /// exactly `target` bytes.
    pub fn new(dec: D, target: usize) -> DecodeExact<D> {
        DecodeExact {
            target,
            read: 0,
            dec,
        }
    }
}

impl<D> AsyncDecode for DecodeExact<D>
    where D: AsyncDecode
{
    type Item = D::Item;
    type Error = DecodeExactError<D::Error, D::Item>;

    fn poll_decode<R: AsyncRead>(mut self,
                                 cx: &mut Context,
                                 reader: &mut R)
                                 -> PollDec<Self::Item, Self, Self::Error> {
        match self.dec
                  .poll_decode(cx, &mut LimitedReader::new(reader, self.target - self.read)) {
            Done(item, read) => {
                self.read += read;
                debug_assert!(self.read <= self.target);

                if self.read < self.target {
                    Errored(DecodeError::DataError(DecodeExactError::Early(item, self.read)))
                } else {
                    Done(item, read)
                }
            }
            Progress(inner, read) => {
                self.read += read;
                debug_assert!(self.read <= self.target);

                self.dec = inner;
                Progress(self, read)
            }
            Pending(inner) => {
                self.dec = inner;
                Pending(self)
            }
            Errored(err) => {
                match err {
                    DecodeError::DataError(inner_err) => {
                        Errored(DecodeError::DataError(inner_err.into()))
                    }
                    DecodeError::ReaderError(inner_err) => Errored(inner_err.into()),
                }
            }
        }
    }
}
