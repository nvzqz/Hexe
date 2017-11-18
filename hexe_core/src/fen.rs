//! [Forsyth–Edwards Notation][fen] parsing.
//!
//! [fen]: https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation

use prelude::*;
use piece::map::PieceMap;

/// A type that can used to parse [Forsyth–Edwards Notation (FEN)][fen].
///
/// [fen]: https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
#[derive(Clone, PartialEq, Eq)]
pub struct Fen {
    /// The pieces on the board.
    pub pieces: PieceMap,
    /// The active color.
    pub color: Color,
    /// The castling rights.
    pub castling: CastleRights,
    /// The en passant target square.
    pub en_passant: Option<Square>,
    /// The number of halfmoves since the last capture or pawn advance.
    pub halfmoves: u32,
    /// The fullmove number.
    pub fullmoves: u32,
}
