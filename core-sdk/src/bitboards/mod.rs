use crate::bitboards::bitboards::constants::*;
use crate::bitboards::bitboards::*;
use crate::bitboards::magic_constants::*;
use crate::board_representation::game_state::{
    file_of, rank_of, BLACK, CASTLE_ALL, CASTLE_ALL_BLACK, CASTLE_ALL_WHITE, CASTLE_BLACK_KS, CASTLE_BLACK_QS, CASTLE_WHITE_KS, CASTLE_WHITE_QS, WHITE,
};
use crate::move_generation::magic::Magic;
use crate::move_generation::movegen::{bishop_attack, rook_attack};
use std::fmt::Display;

pub mod bitboards;

//Code for generating bitboards::
pub(crate) fn arr_2d_to_string<T: Display>(arr: &[[T; 64]; 64], name: &str) -> String {
    let mut res_str: String = String::new();
    res_str.push_str(&format!("#[rustfmt::skip]\npub const {} : [[{};64];64]= [", name, std::any::type_name::<T>(),));
    for a in arr.iter() {
        res_str.push_str("[");
        for i in a.iter() {
            res_str.push_str(&format!("{}{}, ", *i, std::any::type_name::<T>()));
        }
        res_str.push_str("], ");
    }
    res_str.push_str("];");
    res_str
}
pub(crate) fn side_arr_to_string<T: Display>(arr: &[[T; 64]; 2], name: &str) -> String {
    let mut res_str: String = String::new();
    res_str.push_str(&format!("#[rustfmt::skip]\npub const {} : [[{};64];2] = [", name, std::any::type_name::<T>()));
    for side in 0..2 {
        res_str.push_str("[");
        for i in arr[side].iter() {
            res_str.push_str(&format!("{}{}, ", *i, std::any::type_name::<T>()));
        }
        res_str.push_str("],");
    }
    res_str.push_str("];");
    res_str
}
pub(crate) fn arr_to_string<T: Display>(arr: &[T], name: &str) -> String {
    let mut res_str: String = String::new();
    res_str.push_str(&format!("#[rustfmt::skip]\npub const {} : [{};{}] = [", name, std::any::type_name::<T>(), arr.len()));
    for i in arr {
        res_str.push_str(&format!("{}{}, ", *i, std::any::type_name::<T>()));
    }
    res_str.push_str("];");
    res_str
}

pub(crate) fn magic_arr_to_string(arr: &[Magic], name: &str) -> String {
    let mut res_str = String::new();
    res_str.push_str(&format!("#[rustfmt::skip]\npub const {}: [Magic;{}] = [", name, arr.len()));
    for i in arr {
        res_str.push_str(&format!("{}, ", *i));
    }
    res_str.push_str("];");
    res_str
}
pub fn print_magics() {
    let mut res = Vec::with_capacity(0);
    let mut previous_offset = 0;
    for sq in 0..64 {
        let mask = OCCUPANCY_MASKS_ROOK[sq];
        res.push(Magic {
            occupancy_mask: mask,
            shift: mask.count_ones() as usize,
            magic: MAGICS_ROOK[sq],
            offset: previous_offset,
        });
        previous_offset += 1 << OCCUPANCY_MASKS_ROOK[sq].count_ones();
    }
    println!("{}", magic_arr_to_string(&res, "MAGIC_ROOK"));
    println!("Offset: {}", previous_offset);
    let mut res = Vec::with_capacity(0);
    for sq in 0..64 {
        let mask = OCCUPANCY_MASKS_BISHOP[sq];
        res.push(Magic {
            occupancy_mask: mask,
            shift: mask.count_ones() as usize,
            magic: MAGICS_BISHOP[sq],
            offset: previous_offset,
        });
        previous_offset += 1 << OCCUPANCY_MASKS_BISHOP[sq].count_ones();
    }
    println!("{}", magic_arr_to_string(&res, "MAGIC_BISHOP"));
    println!("Offset: {}", previous_offset);
}

pub fn print_castle_permisssion() {
    let mut res = [CASTLE_ALL; 64];
    res[square::E1] &= !CASTLE_ALL_WHITE;
    res[square::A1] &= !CASTLE_WHITE_QS;
    res[square::H1] &= !CASTLE_WHITE_KS;
    res[square::E8] &= !CASTLE_ALL_BLACK;
    res[square::A8] &= !CASTLE_BLACK_QS;
    res[square::H8] &= !CASTLE_BLACK_KS;
    println!("{}", arr_to_string(&res, "CASTLE_PERMISSION"));
}

pub const fn occupancy_mask_rook(square: usize) -> u64 {
    ((RANKS[rank_of(square)] & !(FILES[0] | FILES[7])) | (FILES[file_of(square)] & !(RANKS[0] | RANKS[7]))) & not_square(square)
}

pub fn print_rook_occupancy_masks() {
    let mut res = [0u64; 64];
    for sq in 0..64 {
        res[sq] = occupancy_mask_rook(sq);
    }
    println!("{}", arr_to_string(&res, "OCCUPANCY_MASKS_ROOK"));
}

pub fn occupancy_mask_bishops(square: usize) -> u64 {
    let mut res = 0u64;
    let rk = rank_of(square) as isize;
    let fl = file_of(square) as isize;
    let dirs: [(isize, isize); 4] = [(1, 1), (-1, -1), (1, -1), (-1, 1)];
    for dir in dirs.iter() {
        let (file_i, rank_i) = dir;
        let mut rn = rk + rank_i;
        let mut fnn = fl + file_i;
        while rn >= 1 && rn <= 6 && fnn >= 1 && fnn <= 6 {
            res |= 1u64 << (rn * 8 + fnn);
            rn += rank_i;
            fnn += file_i;
        }
    }
    res
}

pub fn print_bishop_occupancy_masks() {
    let mut res = [0u64; 64];
    for sq in 0..64 {
        res[sq] = occupancy_mask_bishops(sq);
    }
    println!("{}", arr_to_string(&res, "OCCUPANCY_MASKS_BISHOP"))
}

pub fn print_bishop_rays() {
    let mut res = [[0u64; 64]; 64];
    for king_sq in 0..64 {
        for bishop_sq in 0..64 {
            res[king_sq][bishop_sq] = get_bishop_ray_slow(FREEFIELD_BISHOP_ATTACKS[king_sq], king_sq, bishop_sq);
        }
    }
    println!("{}", arr_2d_to_string(&res, "BISHOP_RAYS"))
}

//Gets the ray of one bishop into a specific direction
pub fn get_bishop_ray_slow(bishop_attack_in_all_directions: u64, target_square: usize, bishop_square: usize) -> u64 {
    let diff = target_square as isize - bishop_square as isize;
    let target_rank = rank_of(target_square);
    let target_file = file_of(target_square);
    let bishop_rank = rank_of(bishop_square);
    let bishop_file = file_of(bishop_square);
    if diff > 0 {
        if diff % 9 == 0 {
            FILES_LESS_THAN[target_file] & FILES_GREATER_THAN[bishop_file] & RANKS_LESS_THAN[target_rank] & RANKS_GREATER_THAN[bishop_rank] & bishop_attack_in_all_directions
        } else {
            FILES_GREATER_THAN[target_file] & FILES_LESS_THAN[bishop_file] & RANKS_LESS_THAN[target_rank] & RANKS_GREATER_THAN[bishop_rank] & bishop_attack_in_all_directions
        }
    } else if diff % -9 == 0 {
        FILES_GREATER_THAN[target_file] & FILES_LESS_THAN[bishop_file] & RANKS_GREATER_THAN[target_rank] & RANKS_LESS_THAN[bishop_rank] & bishop_attack_in_all_directions
    } else {
        FILES_LESS_THAN[target_file] & FILES_GREATER_THAN[bishop_file] & RANKS_GREATER_THAN[target_rank] & RANKS_LESS_THAN[bishop_rank] & bishop_attack_in_all_directions
    }
}

pub fn print_rook_rays() {
    let mut res = [[0u64; 64]; 64];
    for king_sq in 0..64 {
        for rook_sq in 0..64 {
            res[king_sq][rook_sq] = get_rook_ray_slow(FREEFIELD_ROOK_ATTACKS[king_sq], king_sq, rook_sq);
        }
    }
    println!("{}", arr_2d_to_string(&res, "ROOK_RAYS"));
}

//Gets the ray of one rook into a specific direction
pub fn get_rook_ray_slow(rook_attacks_in_all_directions: u64, target_square: usize, rook_square: usize) -> u64 {
    let diff = target_square as isize - rook_square as isize;
    let target_rank = rank_of(target_square);
    let target_file = file_of(target_square);
    let rook_rank = rank_of(rook_square);
    let rook_file = file_of(rook_square);
    if diff > 0 {
        //Same vertical
        if target_rank == rook_rank {
            FILES_LESS_THAN[target_file] & FILES_GREATER_THAN[rook_file] & rook_attacks_in_all_directions
        } else {
            RANKS_LESS_THAN[target_rank] & RANKS_GREATER_THAN[rook_rank] & rook_attacks_in_all_directions
        }
    } else if target_rank == rook_rank {
        FILES_GREATER_THAN[target_file] & FILES_LESS_THAN[rook_file] & rook_attacks_in_all_directions
    } else {
        RANKS_GREATER_THAN[target_rank] & RANKS_LESS_THAN[rook_rank] & rook_attacks_in_all_directions
    }
}
pub fn print_king_zone() {
    let mut res = [0u64; 64];
    for king_sq in 0..64 {
        let zone = 1u64 << king_sq | KING_ATTACKS[king_sq];
        res[king_sq] = zone | north_one(zone) | south_one(zone);
        if file_of(king_sq) == 0 {
            res[king_sq] |= east_one(res[king_sq]);
        } else if file_of(king_sq) == 7 {
            res[king_sq] |= west_one(res[king_sq]);
        }
    }
    println!("{}", arr_to_string(&res, "KING_ZONE"))
}
pub fn print_freefield_rook_attacks() {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        *item = rook_attack(sq, 0u64);
    }
    println!("{}", arr_to_string(&res, "FREEFIELD_ROOK_ATTACKS"))
}
pub const fn init_freefield_bishop_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        //res[sq] = bishop_attack()
        sq += 1;
    }
    res
}
pub fn print_freefield_bishop_attacks() {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        *item = bishop_attack(sq, 0u64);
    }
    println!("{}", arr_to_string(&res, "FREEFIELD_BISHOP_ATTACKS"))
}

pub const fn init_shielding_pawns() -> [[u64; 64]; 2] {
    let mut res = [[0u64; 64]; 2];
    let mut sq = 0;
    while sq < 64 {
        let king = square(sq);
        let shield = north_one(king) | north_west_one(king) | north_east_one(king);
        res[WHITE][sq] = shield | north_one(shield);
        let shield = south_one(king) | south_west_one(king) | south_east_one(king);
        res[BLACK][sq] = shield | south_one(shield);
        sq += 1;
    }
    let mut rank = 0;
    while rank < 8 {
        let mut side = 0;
        while side < 2 {
            res[side][8 * rank] = res[side][8 * rank + 1];
            res[side][8 * rank + 7] = res[side][8 * rank + 6];
            side += 1;
        }
        rank += 1;
    }
    res
}

pub const fn init_diagonally_adjacent() -> [u64; 64] {
    let mut res = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        res[sq] = north_east_one(square(sq)) | north_west_one(square(sq)) | south_east_one(square(sq)) | south_west_one(square(sq));
        sq += 1;
    }
    res
}

pub const fn init_files_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut file = 0;
    while file < 8 {
        let mut file_less_than = 0;
        while file_less_than < file {
            res[file] |= FILES[file_less_than];
            file_less_than += 1;
        }
        file += 1;
    }
    res
}

pub const fn init_ranks_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut rank = 0;
    while rank < 8 {
        let mut rank_less_than = 0;
        while rank_less_than < rank {
            res[rank] |= RANKS[rank_less_than];
            rank_less_than += 1;
        }
        rank += 1;
    }
    res
}

pub const fn init_files_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut file = 0;
    while file < 8 {
        res[file] = !FILES_LESS_THAN[file] & !FILES[file];
        file += 1;
    }
    res
}

pub const fn init_ranks_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut rank = 0;
    while rank < 8 {
        res[rank] = !RANKS_LESS_THAN[rank] & !RANKS[rank];
        rank += 1;
    }
    res
}

const fn king_attack(mut king_board: u64) -> u64 {
    let mut attacks = east_one(king_board) | west_one(king_board);
    king_board |= attacks;
    attacks |= south_one(king_board) | north_one(king_board);
    attacks
}

pub const fn init_king_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        res[sq] = king_attack(1u64 << sq);
        sq += 1;
    }
    res
}

const fn knight_attack(knight: u64) -> u64 {
    let mut attacks;
    let mut east = east_one(knight);
    let mut west = west_one(knight);
    attacks = (east | west) << 16;
    attacks |= (east | west) >> 16;
    east = east_one(east);
    west = west_one(west);
    attacks |= (east | west) << 8;
    attacks |= (east | west) >> 8;
    attacks
}

pub const fn init_knight_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        res[sq] = knight_attack(1u64 << sq);
        sq += 1;
    }
    res
}

pub const fn init_ranks() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut rank = 0;
    while rank < 8 {
        if rank == 0 {
            res[0] = 1u64 | 1u64 << 1 | 1u64 << 2 | 1u64 << 3 | 1u64 << 4 | 1u64 << 5 | 1u64 << 6 | 1u64 << 7;
        } else {
            res[rank] = res[rank - 1] << 8;
        }
        rank += 1;
    }
    res
}

pub const fn init_files() -> [u64; 8] {
    let mut res = [0u64; 8];
    let mut file = 0;
    while file < 8 {
        if file == 0 {
            res[0] = 1u64 | 1u64 << 8 | 1u64 << 16 | 1u64 << 24 | 1u64 << 32 | 1u64 << 40 | 1u64 << 48 | 1u64 << 56;
        } else {
            res[file] = res[file - 1] << 1;
        }
        file += 1;
    }
    res
}
