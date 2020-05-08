use rand::prelude::*;
use std::u64;

pub fn rand_u64(rng: &mut StdRng) -> u64 {
    let res: u64 = rng.gen();
    res
}
lazy_static! {
    pub static ref ZOBRIST_KEYS: Zobrist = init_zobrist();
}
pub fn init_at_program_start() {
    ZOBRIST_KEYS.w_pawns.len();
}

pub fn rand_array_64(rng: &mut StdRng) -> [u64; 64] {
    let mut res = [0u64; 64];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}
pub fn rand_array_16(rng: &mut StdRng) -> [u64; 16] {
    let mut res = [0u64; 16];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}
pub fn rand_array_8(rng: &mut StdRng) -> [u64; 8] {
    let mut res = [0u64; 8];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn init_zobrist() -> Zobrist {
    let mut generator: StdRng = SeedableRng::from_seed([42; 32]);
    let w_pawns = rand_array_64(&mut generator);
    let w_knights = rand_array_64(&mut generator);
    let w_bishops = rand_array_64(&mut generator);
    let w_rooks = rand_array_64(&mut generator);
    let w_queens = rand_array_64(&mut generator);
    let w_king = rand_array_64(&mut generator);
    let b_pawns = rand_array_64(&mut generator);
    let b_knights = rand_array_64(&mut generator);
    let b_bishops = rand_array_64(&mut generator);
    let b_rooks = rand_array_64(&mut generator);
    let b_queens = rand_array_64(&mut generator);
    let b_king = rand_array_64(&mut generator);
    let side_to_move = rand_u64(&mut generator);
    let en_passant = rand_array_8(&mut generator);
    let castle_permissions = rand_array_16(&mut generator);
    Zobrist {
        w_pawns,
        w_knights,
        w_bishops,
        w_rooks,
        w_queens,
        w_king,
        b_pawns,
        b_knights,
        b_bishops,
        b_rooks,
        b_queens,
        b_king,
        side_to_move,
        en_passant,
        castle_permissions,
    }
}

pub struct Zobrist {
    pub w_pawns: [u64; 64],
    pub w_knights: [u64; 64],
    pub w_bishops: [u64; 64],
    pub w_rooks: [u64; 64],
    pub w_queens: [u64; 64],
    pub w_king: [u64; 64],
    pub b_pawns: [u64; 64],
    pub b_knights: [u64; 64],
    pub b_bishops: [u64; 64],
    pub b_rooks: [u64; 64],
    pub b_queens: [u64; 64],
    pub b_king: [u64; 64],
    pub side_to_move: u64,
    pub castle_permissions: [u64; 16],
    pub en_passant: [u64; 8],
}
