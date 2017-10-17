static RAY_ATTACKS: [u64; TABLE_SIZE] = [
];

const TABLE_SIZE: usize = NUM_BISHOP_ATTACKS + NUM_ROOK_ATTACKS;

const NUM_ROOK_ATTACKS: usize = 102400;
const NUM_BISHOP_ATTACKS: usize = 5248;

pub fn rook_attacks() -> &'static [u64] {
    &RAY_ATTACKS[NUM_BISHOP_ATTACKS..]
}

pub fn bishop_attacks() -> &'static [u64] {
    &RAY_ATTACKS[..NUM_BISHOP_ATTACKS]
}