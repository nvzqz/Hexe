//! A chess board square and its components.
//!
//! A chess board is comprised of sixty-four squares ranging from A1 through H8.
//! Each square has two components:
//!
//! - File: a column represented by a letter from A through F
//! - Rank: a row represented by a number from 1 through 8
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use hexe_core::square::{Square, File, Rank};
//!
//! let f = File::B;
//! let r = Rank::Seven;
//! let sq = Square::B7;
//!
//! assert_eq!(sq, Square::new(f, r));
//! ```
//!
//! [`Square`] is an `enum` so that we can safely and conveniently index into
//! tables of sixty-four elements. Because the optimizer knows that the index
//! will **never** be greater than 64, the bounds check gets removed, thus
//! making lookups fast.
//!
//! ```
//! # use hexe_core::prelude::*;
//! # type T = ();
//! static TABLE: [T; 64] = [
//!     /* ... */
//! #   (); 64
//! ];
//!
//! pub fn get_value(sq: Square) -> T {
//!     // Will never panic
//!     TABLE[sq as usize]
//! }
//! ```
//!
//! [`Square`]: enum.Square.html

use core::{fmt, ops, str};
use prelude::*;
use uncon::*;

#[cfg(feature = "serde")]
use serde::*;

#[cfg(all(test, nightly))]
mod benches;

#[cfg(test)]
mod tests;

mod tables;
use self::tables::TABLES;

impl_ord!(Square, File, Rank);
impl_rand!(u8 => Square, File, Rank);

/// A square on a chess board.
#[derive(Copy, Clone, Hash, PartialEq, Eq, FromUnchecked)]
#[uncon(impl_from, other(u16, u32, u64, usize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl fmt::Debug for Square {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Square {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.map_str(|s| s.fmt(f))
    }
}

impl From<(File, Rank)> for Square {
    #[inline]
    fn from((file, rank): (File, Rank)) -> Square {
        Square::new(file, rank)
    }
}

define_from_str_error! { Square;
    /// The error returned when `Square::from_str` fails.
    "failed to parse a string as a square"
}

impl str::FromStr for Square {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Square, FromStrError> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 { Err(FromStrError(())) } else {
            // Gets better optimized as a macro for some strange reason
            macro_rules! convert {
                ($lo:expr, $hi:expr, $b:expr) => {
                    match $b {
                        $lo...$hi => unsafe { ($b - $lo).into_unchecked() },
                        _ => return Err(FromStrError(())),
                    }
                }
            }
            Ok(Square::new(convert!(b'a', b'h', bytes[0] | 32),
                           convert!(b'1', b'8', bytes[1])))
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Square {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.map_str(|s| ser.serialize_str(s))
    }
}

const FILE_BITS: u8 = 7;
const RANK_BITS: u8 = FILE_BITS << RANK_SHIFT;
const RANK_SHIFT: usize = 3;

const TRIANGLE_LEN: usize = 64 * 65 / 2;

impl Square {
    /// Initializes a `Square` from a `File` and `Rank`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::square::*;
    /// let s = Square::new(File::B, Rank::Five);
    ///
    /// assert_eq!(s.file(), File::B);
    /// assert_eq!(s.rank(), Rank::Five);
    /// ```
    #[inline]
    pub fn new(file: File, rank: Rank) -> Square {
        (((rank as u8) << RANK_SHIFT) | (file as u8)).into()
    }

    #[inline]
    pub(crate) fn between(self, other: Square) -> Bitboard {
        Bitboard(TABLES.between[self as usize][other as usize])
    }

    #[inline]
    pub(crate) fn line(self, other: Square) -> Bitboard {
        Bitboard(TABLES.line[self as usize][other as usize])
    }

    /// Returns the `File` for `self`.
    #[inline]
    pub fn file(self) -> File {
        ((self as u8) & FILE_BITS).into()
    }

    /// Returns the `Rank` for `self`.
    #[inline]
    pub fn rank(self) -> Rank {
        ((self as u8) >> RANK_SHIFT).into()
    }

    /// Reverses the file of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// assert_eq!(Square::B2.rev_file(), Square::G2);
    /// ```
    #[inline]
    pub fn rev_file(self) -> Square {
        (FILE_BITS ^ self as u8).into()
    }

    /// Reverses the rank of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// assert_eq!(Square::B2.rev_rank(), Square::B7);
    /// ```
    #[inline]
    pub fn rev_rank(self) -> Square {
        (RANK_BITS ^ self as u8).into()
    }

    /// Returns `self` shifted up one rank, or `None` if at last rank.
    #[inline]
    pub fn up(self) -> Option<Square> {
        match self.rank() {
            Rank::Eight => None,
            _ => {
                let val = self as u8 + (1 << RANK_SHIFT);
                Some(unsafe { val.into_unchecked() })
            }
        }
    }

    /// Returns `self` shifted down one rank, or `None` if at first rank.
    #[inline]
    pub fn down(self) -> Option<Square> {
        match self.rank() {
            Rank::One => None,
            _ => {
                let val = self as u8 - (1 << RANK_SHIFT);
                Some(unsafe { val.into_unchecked() })
            }
        }
    }

    /// Returns `self` shifted right one file, or `None` if at last file.
    #[inline]
    pub fn right(self) -> Option<Square> {
        match self.file() {
            File::H => None,
            _ => {
                let val = self as u8 + 1;
                Some(unsafe { val.into_unchecked() })
            }
        }
    }

    /// Returns `self` shifted left one file, or `None` if at first file.
    #[inline]
    pub fn left(self) -> Option<Square> {
        match self.file() {
            File::A => None,
            _ => {
                let val = self as u8 - 1;
                Some(unsafe { val.into_unchecked() })
            }
        }
    }

    /// Combines the file of `self` with the rank of `other`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let s1 = Square::B5;
    /// let s2 = Square::C7;
    ///
    /// assert_eq!(s1.combine(s2), Square::B7);
    /// assert_eq!(s2.combine(s1), Square::C5);
    /// ```
    #[inline]
    pub fn combine(self, other: Square) -> Square {
        ((FILE_BITS & self as u8) | (RANK_BITS & other as u8)).into()
    }

    /// Returns the `Color` for `self`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let a = Square::A1;
    /// assert_eq!(a.color(), Color::Black);
    ///
    /// let b = Square::B5;
    /// assert_eq!(b.color(), Color::White);
    /// ```
    #[inline]
    pub fn color(self) -> Color {
        const BLACK: usize = Bitboard::BLACK.0 as usize;
        const MOD:   usize = ::consts::PTR_SIZE * 8;
        (BLACK >> (self as usize % MOD)).into()
    }

    /// Returns whether `self` and `other` are equal in color.
    #[inline]
    pub fn color_eq(self, other: Square) -> bool {
        let bits = self as u8 ^ other as u8;
        ((bits >> RANK_SHIFT) ^ bits) & 1 == 0
    }

    /// Returns whether `self` is aligned with two other squares along a file,
    /// rank, or diagonal.
    ///
    /// # Examples
    ///
    /// Square A3 lies on the same diagonal as C5 and F8:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// assert!(Square::A3.is_aligned(Square::C5, Square::F8));
    /// ```
    #[inline]
    pub fn is_aligned(self, a: Square, b: Square) -> bool {
        a.line(b).contains(self)
    }

    /// Returns whether `self` is between two other squares along a file, rank,
    /// or diagonal.
    ///
    /// # Examples
    ///
    /// Square D4 lies between B2 and G7 along a diagonal:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// assert!(Square::D4.is_between(Square::B2, Square::G7));
    /// ```
    #[inline]
    pub fn is_between(self, a: Square, b: Square) -> bool {
        a.between(b).contains(self)
    }

    /// Calculates the [Chebyshev distance][wiki] between `self` and `other`.
    ///
    /// The result is the number of steps required to move a king from one
    /// square to the other.
    ///
    /// # Examples
    ///
    /// It takes a king two moves to travel the same distance as a knight:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// for s1 in Square::ALL {
    ///     for s2 in s1.knight_attacks() {
    ///         assert_eq!(s1.distance(s2), 2);
    ///     }
    /// }
    /// ```
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Chebyshev_distance
    #[inline]
    pub fn distance(self, other: Square) -> usize {
        TABLES.distance[self as usize][other as usize] as usize
    }

    /// Calculates the [Manhattan distance][wiki] between `self` and `other`.
    ///
    /// The result is the distance when strictly taking a horizontal/vertical
    /// path from one square to the other.
    ///
    /// # Examples
    ///
    /// The knight's path always traverses three squares:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// for s1 in Square::ALL {
    ///     for s2 in s1.knight_attacks() {
    ///         assert_eq!(s1.man_distance(s2), 3);
    ///     }
    /// }
    /// ```
    ///
    /// [wiki]: https://en.wiktionary.org/wiki/Manhattan_distance
    #[inline]
    pub fn man_distance(self, other: Square) -> usize {
        self.file().distance(other.file()) + self.rank().distance(other.rank())
    }

    /// Calculates the [Chebyshev distance][wiki] between `self` and the center
    /// of the board.
    ///
    /// # Examples
    ///
    /// It takes a king three moves to travel to the center of the board from
    /// any edge:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let edges = File::A | File::H | Rank::One | Rank::Eight;
    ///
    /// for sq in edges {
    ///     assert_eq!(sq.center_distance(), 3);
    /// }
    /// ```
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Chebyshev_distance
    #[inline]
    pub fn center_distance(self) -> usize {
        TABLES.chebyshev[self as usize] as usize
    }

    /// Calculates the [Manhattan distance][wiki] between `self` and the center
    /// of the board.
    ///
    /// [wiki]: https://en.wiktionary.org/wiki/Manhattan_distance
    #[inline]
    pub fn center_man_distance(self) -> usize {
        TABLES.manhattan[self as usize] as usize
    }

    /// Returns the [triangular index][wiki] for `self` and `other`.
    ///
    /// This allows indexing into tables of size 2080, which is slightly greater
    /// than 2048 (64 × 64 ÷ 2).
    ///
    /// # Tradeoffs
    /// While this allows for using 50.78% as much memory as a table of size
    /// 4096 (64 × 64), computing the index takes a considerable amount of time
    /// compared to indexing into a 64 × 64 table.
    ///
    /// # Safety
    /// The result index has been thoroughly tested to **always** return a value
    /// less than 2080. Because of this, it is safe to call [`get_unchecked`] on
    /// arrays/slices of that size or greater.
    ///
    /// [wiki]: https://chessprogramming.wikispaces.com/Square+Attacked+By#Legality%20Test-In%20Between-Triangular%20Lookup
    /// [`get_unchecked`]: https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked
    #[inline]
    pub fn tri_index(self, other: Square) -> usize {
        let mut a = self  as isize;
        let mut b = other as isize;
        let mut d = a - b;
        d &= d >> 31;
        b += d;
        a -= d;
        b *= b ^ 127;
        ((b >> 1) + a) as usize
    }

    /// Returns a shared reference to an element from the table via triangular
    /// index.
    #[inline]
    pub fn tri<T>(self, other: Square, table: &[T; TRIANGLE_LEN]) -> &T {
        // tri index < TRIANGLE_LEN
        unsafe { table.get_unchecked(self.tri_index(other)) }
    }

    /// Returns a mutable reference to an element from the table via triangular
    /// index.
    #[inline]
    pub fn tri_mut<T>(self, other: Square, table: &mut [T; TRIANGLE_LEN]) -> &mut T {
        unsafe { table.get_unchecked_mut(self.tri_index(other)) }
    }

    /// Returns the result of applying a function to a mutable string
    /// representation of `self`.
    ///
    /// This is a _much_ preferred way of getting the string representation of
    /// a square, especially in when using `#![no_std]`. The alternative would
    /// be to use `to_string` or `format!`, which perform a heap allocation
    /// whereas this uses a stack-allocated string.
    ///
    /// # Examples
    ///
    /// The string's lifetime is for the duration of the closure's execution:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// Square::A5.map_str(|s| assert_eq!(s, "A5"));
    /// ```
    #[inline]
    pub fn map_str<T, F: FnOnce(&mut str) -> T>(self, f: F) -> T {
        let mut buf = [char::from(self.file()) as u8,
                       char::from(self.rank()) as u8];
        unsafe { f(str::from_utf8_unchecked_mut(&mut buf)) }
    }

    /// Returns the pawn attacks for `self` and `color`.
    #[inline]
    pub fn pawn_attacks(self, color: Color) -> Bitboard {
        Bitboard(TABLES.pawns[color as usize][self as usize])
    }

    /// Returns the knight attacks for `self`.
    #[inline]
    pub fn knight_attacks(self) -> Bitboard {
        Bitboard(TABLES.knight[self as usize])
    }

    /// Returns the rook attacks for `self` and `occupied`.
    ///
    /// Whether or not `occupied` contains `self` does not matter.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let start = Square::A1;
    ///
    /// let occ = Square::A3 | Square::C1;
    /// let exp = Square::A2 | Square::B1 | occ;
    ///
    /// assert_eq!(start.rook_attacks(occ), exp);
    /// ```
    #[inline]
    pub fn rook_attacks(self, occupied: Bitboard) -> Bitboard {
        ::magic::rook_attacks(self, occupied)
    }

    /// Returns the bishop attacks for `self` and `occupied`.
    ///
    /// Whether or not `occupied` contains `self` does not matter.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let start = Square::A1;
    ///
    /// let occ = Square::C3;
    /// let exp = Square::B2 | occ;
    ///
    /// assert_eq!(start.bishop_attacks(occ.into()), exp);
    /// ```
    #[inline]
    pub fn bishop_attacks(self, occupied: Bitboard) -> Bitboard {
        ::magic::bishop_attacks(self, occupied)
    }

    /// Returns the king attacks for `self`.
    #[inline]
    pub fn king_attacks(self) -> Bitboard {
        Bitboard(TABLES.king[self as usize])
    }

    /// Returns the queen attacks for `self` and `occupied`.
    ///
    /// This works the same as combining the results of `rook_attacks` and
    /// `bishop_attacks`.
    #[inline]
    pub fn queen_attacks(self, occupied: Bitboard) -> Bitboard {
        self.rook_attacks(occupied) | self.bishop_attacks(occupied)
    }
}

/// A file (or column) for a chess board.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, FromUnchecked)]
#[uncon(impl_from, other(u16, u32, u64, usize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum File { A, B, C, D, E, F, G, H }

impl File {
    /// Returns a file from the parsed character.
    #[inline]
    pub fn from_char(ch: char) -> Option<File> {
        match 32 | ch as u8 {
            b @ b'a' ... b'f' => unsafe {
                Some((b - b'a').into_unchecked())
            },
            _ => None,
        }
    }

    /// Returns the adjacent mask for `self`, containing all squares on the
    /// files directly to the left and right of `self`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let val = File::C;
    /// let adj = File::B | File::D;
    ///
    /// assert_eq!(val.adjacent_mask(), adj);
    /// ```
    #[inline]
    pub fn adjacent_mask(&self) -> Bitboard {
        Bitboard(TABLES.adj_file[*self as usize])
    }
}

/// A rank (or row) for a chess board.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, FromUnchecked)]
#[uncon(impl_from, other(u16, u32, u64, usize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Rank { One, Two, Three, Four, Five, Six, Seven, Eight }

impl Rank {
    /// Returns the first rank for `color`.
    #[inline]
    pub fn first(color: Color) -> Rank {
        match color {
            Color::White => Rank::One,
            Color::Black => Rank::Eight,
        }
    }

    /// Returns the last rank for `color`.
    #[inline]
    pub fn last(color: Color) -> Rank {
        Rank::first(!color)
    }

    /// Returns a rank from the parsed character.
    #[inline]
    pub fn from_char(ch: char) -> Option<Rank> {
        match ch as u8 {
            b @ b'1' ... b'8' => unsafe {
                Some((b - b'1').into_unchecked())
            },
            _ => None,
        }
    }

    /// Returns the adjacent mask for `self`, containing all squares on the
    /// ranks directly ahead and behind `self`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let val = Rank::Five;
    /// let adj = Rank::Four | Rank::Six;
    ///
    /// assert_eq!(val.adjacent_mask(), adj);
    /// ```
    #[inline]
    pub fn adjacent_mask(&self) -> Bitboard {
        Bitboard(TABLES.adj_rank[*self as usize])
    }

    /// Returns the remaining distance for `color` to reach the end of the board
    /// from `self`.
    ///
    /// This is useful for finding the number of moves a pawn must make to be
    /// promoted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hexe_core::prelude::*;
    /// let rank = Rank::Three;
    ///
    /// assert_eq!(rank.rem_distance(Color::White), 5);
    /// assert_eq!(rank.rem_distance(Color::Black), 2);
    /// ```
    #[inline]
    pub fn rem_distance(self, color: Color) -> usize {
        (0b111 * !color as usize) ^ self as usize
    }
}

macro_rules! impl_components {
    ($($t:ty, $c:expr, $m:expr;)+) => { $(
        impl From<$t> for char {
            #[inline]
            fn from(val: $t) -> char {
                ($c + val as u8) as char
            }
        }

        impl ops::Not for $t {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                (7 - self as u8).into()
            }
        }

        #[cfg(feature = "serde")]
        impl Serialize for $t {
            fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                ser.serialize_char((*self).into())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                Self::from_char(char::deserialize(de)?).ok_or_else(|| {
                    de::Error::custom($m)
                })
            }
        }

        impl $t {
            /// Returns the distance between `self` and `other`.
            #[inline]
            pub fn distance(self, other: Self) -> usize {
                (self as isize - other as isize).abs() as usize
            }
        }
    )+ }
}

impl_components! {
    File, b'A', "failed to parse board file";
    Rank, b'1', "failed to parse board rank";
}
