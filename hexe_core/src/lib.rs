//! # Hexe Core
//!
//! This crate defines the building blocks for the Hexe chess engine.

#![doc(html_logo_url = "https://raw.githubusercontent.com/hexe-rs/Hexe/assets/Icon.png")]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(test)]
extern crate rand;

#[macro_use]
extern crate uncon_derive;
extern crate uncon;

pub mod prelude;

pub mod bitboard;
pub mod castle_rights;
pub mod color;
pub mod square;

mod magic;
