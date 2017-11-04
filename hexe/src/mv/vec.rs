//! An inline vector of moves.

use std::mem;
use std::u8;

const VEC_CAP: usize = u8::MAX as usize;

/// An inline vector of moves generated by a `Position`.
///
/// There is no known case where there have been more than 255 moves for a legal
/// position. Because of this, performing an allocation for a list of generated
/// moves is an avoidable waste of time.
pub struct MoveVec {
    /// The internal inline buffer. Uses u16 for convenience.
    buf: [u16; VEC_CAP],
    /// The vector's length.
    len: u8,
}

impl PartialEq for MoveVec {
    fn eq(&self, other: &MoveVec) -> bool {
        self.len     == other.len &&
        self.buf[..] == other.buf[..]
    }
}

impl Eq for MoveVec {}

impl Clone for MoveVec {
    #[inline]
    fn clone(&self) -> MoveVec {
        MoveVec { buf: self.buf, len: self.len }
    }
}

impl Default for MoveVec {
    #[inline]
    fn default() -> Self {
        MoveVec {
            buf: unsafe { mem::uninitialized() },
            len: 0,
        }
    }
}

impl MoveVec {
    /// Creates a new empty vector.
    #[inline]
    pub fn new() -> MoveVec {
        MoveVec::default()
    }
}
