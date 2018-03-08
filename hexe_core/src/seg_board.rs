//! A bitboard-segmented chess board representations.

use core::{ops, mem};

use bitboard::Bitboard;
use color::Color;
use piece::PieceKind;

const NUM_PIECES: usize = 6;
const NUM_COLORS: usize = 2;

mod values {
    use super::*;

    const PAWN:   u64 = 0x00FF00000000FF00;
    const KNIGHT: u64 = 0x4200000000000042;
    const BISHOP: u64 = 0x2400000000000024;
    const ROOK:   u64 = 0x8100000000000081;
    const QUEEN:  u64 = 0x0800000000000008;
    const KING:   u64 = 0x1000000000000010;
    const WHITE:  u64 = 0x000000000000FFFF;
    const BLACK:  u64 = 0xFFFF000000000000;

    pub const STANDARD: SegBoard = SegBoard {
        pieces: [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING],
        colors: [WHITE, BLACK],
    };
}

/// A full chess board, represented as multiple bitboard segments.
#[repr(C)]
#[derive(Clone)]
pub struct SegBoard {
    pieces: [u64; NUM_PIECES],
    colors: [u64; NUM_COLORS],
}

impl Default for SegBoard {
    #[inline]
    fn default() -> SegBoard {
        unsafe { mem::zeroed() }
    }
}

impl ops::Index<PieceKind> for SegBoard {
    type Output = Bitboard;

    #[inline]
    fn index(&self, kind: PieceKind) -> &Bitboard {
        Bitboard::convert_ref(&self.pieces[kind as usize])
    }
}

impl ops::IndexMut<PieceKind> for SegBoard {
    #[inline]
    fn index_mut(&mut self, kind: PieceKind) -> &mut Bitboard {
        Bitboard::convert_mut(&mut self.pieces[kind as usize])
    }
}

impl ops::Index<Color> for SegBoard {
    type Output = Bitboard;

    #[inline]
    fn index(&self, color: Color) -> &Bitboard {
        Bitboard::convert_ref(&self.colors[color as usize])
    }
}

impl ops::IndexMut<Color> for SegBoard {
    #[inline]
    fn index_mut(&mut self, color: Color) -> &mut Bitboard {
        Bitboard::convert_mut(&mut self.colors[color as usize])
    }
}

impl SegBoard {
    /// The board for standard chess.
    pub const STANDARD: SegBoard = values::STANDARD;

    /// Returns references to the underlying bitboards for `Color` and
    /// `PieceKind`, respectively.
    #[inline]
    pub fn split(&self) -> (&[Bitboard; NUM_COLORS], &[Bitboard; NUM_PIECES]) {
        let colors = &self.colors as *const _ as *const _;
        let pieces = &self.pieces as *const _ as *const _;
        unsafe { (&*colors, &*pieces) }
    }

    /// Returns mutable references to the underlying bitboards for `Color` and
    /// `PieceKind`, respectively.
    #[inline]
    pub fn split_mut(&mut self) -> (&mut [Bitboard; NUM_COLORS], &mut [Bitboard; NUM_PIECES]) {
        let colors = &mut self.colors as *mut _ as *mut _;
        let pieces = &mut self.pieces as *mut _ as *mut _;
        unsafe { (&mut *colors, &mut *pieces) }
    }
}
