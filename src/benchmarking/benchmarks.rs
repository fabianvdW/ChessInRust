#![feature(test)]
extern crate core;
extern crate rand;
extern crate test;

use core::board_representation::game_state::GameState;
use core::testing::openings::load_db_until;
use rand::Rng;
use std::fs;

pub const BENCHMARKING_POSITIONS: &str = "benchmarking_positions.txt";
pub const BENCHMARKING_POSITIONS_AMOUNT: usize = 1000;
pub const MAKE_BENCHMARKING_POSITIONS_FROM: &str = "./O-Deville/o-deville.pgn";
pub const LOAD_DB_UNTIL: usize = 40;

//************************************************************
//* Benchmarking positions are just some random 100 positions from o-deville database loaded until 40th ply
//*
//************************************************************
fn main() {
    make_benchmarking_positions();
}

pub fn make_benchmarking_positions() {
    let mut states: Vec<GameState> =
        load_db_until(MAKE_BENCHMARKING_POSITIONS_FROM, LOAD_DB_UNTIL).0;
    let mut rng = rand::thread_rng();
    let mut write_str = String::new();
    for _ in 0..BENCHMARKING_POSITIONS_AMOUNT {
        let index = rng.gen_range(0, states.len());
        let state = states.remove(index);
        write_str.push_str(&format!("{}\n", state.to_fen()));
    }
    fs::write(BENCHMARKING_POSITIONS, write_str).expect("Unable to write file!");
}

pub fn load_benchmarking_positions() -> Vec<GameState> {
    let mut states = Vec::with_capacity(BENCHMARKING_POSITIONS_AMOUNT);
    let positions =
        fs::read_to_string(BENCHMARKING_POSITIONS).expect("Unable to read benchmarking positions");
    let new_linesplit = positions.split("\n").collect::<Vec<&str>>();
    for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
        states.push(GameState::from_fen(new_linesplit[i]));
    }
    states
}

#[cfg(test)]
mod tests {
    use super::load_benchmarking_positions;
    use super::BENCHMARKING_POSITIONS_AMOUNT;
    use core::board_representation::game_state_attack_container::GameStateAttackContainer;
    use core::evaluation::eval_game_state;
    use core::move_generation::movegen;
    use core::move_generation::movegen::MoveList;
    use core::search::reserved_memory::{ReservedAttackContainer, ReservedMoveList};
    use test::Bencher;

    #[bench]
    pub fn evaluation(b: &mut Bencher) {
        let states = load_benchmarking_positions();
        let mut attack_container = GameStateAttackContainer::default();
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                attack_container.write_state(&states[i]);
                sum += eval_game_state(&states[i], &attack_container, -16000, 16000).final_eval
                    as isize;
            }
            sum
        });
    }
    #[bench]
    pub fn generate_moves(b: &mut Bencher) {
        let states = load_benchmarking_positions();
        let mut attack_container = GameStateAttackContainer::default();
        let mut movelist = MoveList::default();
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                attack_container.write_state(&states[i]);
                movegen::generate_moves(&states[i], false, &mut movelist, &attack_container);
                sum += movelist.move_list.len();
            }
            sum
        });
    }

    #[bench]
    pub fn perft(b: &mut Bencher) {
        let states = load_benchmarking_positions();
        let mut movelist = ReservedMoveList::default();
        let mut attack_container = ReservedAttackContainer::default();
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                sum += core::perft(&states[i], 2, &mut movelist, &mut attack_container);
            }
            sum
        });
    }
}
