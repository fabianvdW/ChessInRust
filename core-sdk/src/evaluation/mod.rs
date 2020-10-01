pub mod parameters;
pub mod params;
pub mod phase;
pub mod psqt_evaluation;
pub mod trace;

use crate::bitboards::bitboards::constants::*;
use crate::bitboards::bitboards::*;
use crate::board_representation::game_state::{file_of, rank_of, relative_rank, GameState, PieceType, BLACK, WHITE};
use crate::move_generation::movegen;
use crate::move_generation::movegen::{pawn_east_targets, pawn_targets, pawn_west_targets};

#[cfg(feature = "tuning")]
use crate::board_representation::game_state::white_pov;
#[cfg(feature = "tuning")]
use crate::evaluation::parameters::normal_parameters::*;
#[cfg(feature = "tuning")]
use crate::evaluation::trace::LargeTrace;
use params::*;
use psqt_evaluation::psqt;
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops;

pub const MG: usize = 0;
pub const EG: usize = 1;

#[derive(Copy, Clone, PartialEq)]
pub struct EvaluationScore(pub i16, pub i16);

impl EvaluationScore {
    pub fn interpolate(self, phase: f32) -> i16 {
        ((f32::from(self.0) * phase + f32::from(self.1) * (128.0 - phase)) / 128.0) as i16
    }
}
impl Default for EvaluationScore {
    fn default() -> Self {
        EvaluationScore(0, 0)
    }
}
impl Display for EvaluationScore {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({} , {})", self.0, self.1)
    }
}
impl Debug for EvaluationScore {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "EvaluationScore({}, {})", self.0, self.1)
    }
}
impl ops::Add<EvaluationScore> for EvaluationScore {
    type Output = EvaluationScore;

    fn add(self, other: EvaluationScore) -> Self::Output {
        EvaluationScore(self.0 + other.0, self.1 + other.1)
    }
}
impl ops::Add<i16> for EvaluationScore {
    type Output = EvaluationScore;

    fn add(self, other: i16) -> Self::Output {
        EvaluationScore(self.0 + other, self.1 + other)
    }
}
impl ops::AddAssign<EvaluationScore> for EvaluationScore {
    fn add_assign(&mut self, other: EvaluationScore) {
        self.0 += other.0;
        self.1 += other.1;
    }
}
impl ops::Sub<EvaluationScore> for EvaluationScore {
    type Output = EvaluationScore;

    fn sub(self, other: EvaluationScore) -> Self::Output {
        EvaluationScore(self.0 - other.0, self.1 - other.1)
    }
}
impl ops::SubAssign<EvaluationScore> for EvaluationScore {
    fn sub_assign(&mut self, other: EvaluationScore) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}
impl ops::Mul<i16> for EvaluationScore {
    type Output = EvaluationScore;

    fn mul(self, other: i16) -> Self::Output {
        EvaluationScore(self.0 * other, self.1 * other)
    }
}
impl ops::MulAssign<i16> for EvaluationScore {
    fn mul_assign(&mut self, other: i16) {
        self.0 *= other;
        self.1 *= other;
    }
}

pub struct EvaluationResult {
    pub final_eval: i16,
    #[cfg(feature = "tuning")]
    pub trace: LargeTrace,
}

pub fn eval_game_state(g: &GameState) -> EvaluationResult {
    #[cfg(feature = "display-eval")]
    {
        println!("Evaluating GameState fen: {}", g.to_fen());
    }
    let mut result = EvaluationResult {
        final_eval: 0,
        #[cfg(feature = "tuning")]
        trace: LargeTrace::default(),
    };
    if is_guaranteed_draw(&g) {
        #[cfg(feature = "tuning")]
        {
            result.trace.is_guaranteed_draw = true;
        }
        return result;
    }
    let mut res = EvaluationScore::default();

    let tempo = if g.get_color_to_move() == WHITE { TEMPO_BONUS } else { TEMPO_BONUS * -1 };
    res += tempo;
    #[cfg(feature = "display-eval")]
    {
        println!("\nTempo:{}", tempo);
    }
    #[cfg(feature = "tuning")]
    {
        result.trace.normal_coeffs[IDX_TEMPO_BONUS] = if g.get_color_to_move() == WHITE { 1 } else { -1 };
    }

    //Initialize all attacks
    let (white_defended_by_minors, white_defended_by_majors) = (g.get_minor_attacks_from_side(WHITE), g.get_major_attacks_from_side(WHITE));
    let white_defended = white_defended_by_minors | white_defended_by_majors | KING_ATTACKS[g.get_king_square(WHITE)];
    let (black_defended_by_minors, black_defended_by_majors) = (g.get_minor_attacks_from_side(BLACK), g.get_major_attacks_from_side(BLACK));
    let black_defended = black_defended_by_minors | black_defended_by_majors | KING_ATTACKS[g.get_king_square(BLACK)];

    let psqt_score: EvaluationScore = if cfg!(feature = "display-eval") || cfg!(feature = "tuning") {
        let (psqt_w, psqt_b) = (
            psqt(
                &g,
                WHITE,
                #[cfg(feature = "tuning")]
                &mut result.trace,
            ),
            psqt(
                &g,
                BLACK,
                #[cfg(feature = "tuning")]
                &mut result.trace,
            ),
        );
        psqt_w - psqt_b
    } else {
        g.get_psqt()
    };
    #[cfg(feature = "display-eval")]
    {
        println!("\nPSQT Sum: {}", psqt_score);
    }
    res += psqt_score;

    let (pieces_w, pieces_b) = (
        piece_values(
            WHITE,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
        piece_values(
            BLACK,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
    );
    #[cfg(feature = "display-eval")]
    {
        println!("\nPiece value Sum: {} - {} -> {}", pieces_w, pieces_b, pieces_w - pieces_b);
    }
    res += pieces_w - pieces_b;

    let (pawns_w, pawns_b) = (
        pawns(
            WHITE,
            g,
            white_defended,
            black_defended,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
        pawns(
            BLACK,
            g,
            black_defended,
            white_defended,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
    );
    #[cfg(feature = "display-eval")]
    {
        println!("\nPawn Sum: {} - {} -> {}", pawns_w, pawns_b, pawns_w - pawns_b);
    }
    res += pawns_w - pawns_b;

    let (knights_w, knights_b) = (
        knights(
            WHITE,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
        knights(
            BLACK,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
    );
    #[cfg(feature = "display-eval")]
    {
        println!("\nKnights Sum: {} - {} -> {}", knights_w, knights_b, knights_w - knights_b);
    }
    res += knights_w - knights_b;

    let (piecewise_w, piecewise_b) = (
        piecewise(
            WHITE,
            g,
            black_defended_by_minors,
            black_defended,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
        piecewise(
            BLACK,
            g,
            white_defended_by_minors,
            white_defended,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
    );
    #[cfg(feature = "display-eval")]
    {
        println!("\nPiecewise Sum: {} - {} -> {}\n", piecewise_w, piecewise_b, piecewise_w - piecewise_b);
    }
    res += piecewise_w - piecewise_b;

    let (king_w, king_b) = (
        king(
            WHITE,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
        king(
            BLACK,
            g,
            #[cfg(feature = "tuning")]
            &mut result.trace,
        ),
    );
    #[cfg(feature = "display-eval")]
    {
        println!("\nKing Sum: {} - {} -> {}", king_w, king_b, king_w - king_b);
    }
    res += king_w - king_b;

    let phase = g.get_phase().phase;
    #[cfg(feature = "tuning")]
    {
        result.trace.phase = phase;
    }
    endgame_rescaling(
        g,
        &mut res,
        phase,
        #[cfg(feature = "tuning")]
        &mut result.trace,
    );

    //Phasing is done the same way stockfish does it
    result.final_eval = res.interpolate(phase);
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nSum: {} + {} + {} + {} + {} + {} + {} -> {} ",
            psqt_score,
            knights_w - knights_b,
            piecewise_w - piecewise_b,
            king_w - king_b,
            pawns_w - pawns_b,
            pieces_w - pieces_b,
            if g.get_color_to_move() == 0 { TEMPO_BONUS } else { TEMPO_BONUS * -1 },
            res
        );
        println!("Phase: {}", phase);
        println!("\nFinal Result: ({} * {} + {} * (128.0 - {}))/128.0 -> {}", res.0, phase, res.1, phase, result.final_eval,);
    }

    result
}

pub fn is_guaranteed_draw(g: &GameState) -> bool {
    if g.get_piece_bb(PieceType::Pawn) | g.get_piece_bb(PieceType::Rook) | g.get_piece_bb(PieceType::Queen) > 0 {
        return false;
    }
    let white_knights = g.get_piece_amt(PieceType::Knight, WHITE);
    let black_knights = g.get_piece_amt(PieceType::Knight, BLACK);
    let white_bishops = g.get_piece_amt(PieceType::Bishop, WHITE);
    let black_bishops = g.get_piece_amt(PieceType::Bishop, BLACK);
    if white_knights + white_bishops <= 2 && black_knights + black_bishops <= 2 {
        if white_knights + white_bishops < 2 || black_knights + black_bishops < 2 {
            if !(white_bishops == 2 && black_bishops == 0) && !(black_bishops == 2 && white_bishops == 0) {
                return true;
            }
        }
    }
    false
}

pub fn endgame_rescaling(g: &GameState, res: &mut EvaluationScore, phase: f32, #[cfg(feature = "tuning")] trace: &mut LargeTrace) {
    let score = res.interpolate(phase);
    let side_ahead = if score >= 0 { WHITE } else { BLACK };
    let side_losing = 1 - side_ahead;
    let winning_pawns = g.get_piece(PieceType::Pawn, side_ahead).count_ones() as usize;
    if winning_pawns <= 1 {
        let losing_minors = (g.get_piece(PieceType::Bishop, side_losing) | g.get_piece(PieceType::Knight, side_losing)).count_ones() as usize;
        let score = score.abs();
        let winnable_ahead = score.abs() >= KNIGHT_PIECE_VALUE.1 + PAWN_PIECE_VALUE.1;

        if !winnable_ahead && (winning_pawns == 0) {
            let factor = SLIGHTLY_WINNING_NO_PAWN;
            *res = EvaluationScore(res.0, (res.1 as f32 * factor) as i16);
            #[cfg(feature = "tuning")]
            {
                trace.slightly_winning_no_pawn = true;
            }
        } else if !winnable_ahead && losing_minors >= 1 && score.abs() + KNIGHT_PIECE_VALUE.1 - PAWN_PIECE_VALUE.1 <= KNIGHT_PIECE_VALUE.1 + PAWN_PIECE_VALUE.1 {
            let factor = SLIGHTLY_WINNING_ENEMY_CAN_SAC;
            *res = EvaluationScore(res.0, (res.1 as f32 * factor) as i16);
            #[cfg(feature = "tuning")]
            {
                trace.slightly_winning_enemy_can_sac = true;
            }
        }
    }
}

pub fn knights(side: usize, g: &GameState, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let mut res = EvaluationScore::default();

    let my_pawn_attacks = pawn_targets(side, g.get_piece(PieceType::Pawn, side));

    let supported_knights = g.get_piece(PieceType::Knight, side) & my_pawn_attacks;
    let supported_knights_amount = supported_knights.count_ones() as i16;
    res += KNIGHT_SUPPORTED_BY_PAWN * supported_knights_amount;
    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_KNIGHT_SUPPORTED] += supported_knights_amount as i8 * if side == WHITE { 1 } else { -1 };
    }
    let mut outpost = EvaluationScore::default();
    let mut _outposts = 0;
    let mut supp = supported_knights;
    while supp != 0u64 {
        let idx = supp.trailing_zeros() as usize;
        supp &= not_file(file_of(idx));
        let mut front_span = pawn_front_span(square(idx), side);
        front_span = west_one(front_span) | east_one(front_span);
        if g.get_piece(PieceType::Pawn, 1 - side) & front_span == 0u64 {
            _outposts += 1;
            outpost += KNIGHT_OUTPOST_TABLE[side][idx];
            #[cfg(feature = "tuning")]
            {
                trace.normal_coeffs[IDX_KNIGHT_OUTPOST_TABLE + white_pov(idx, side)] += if side == WHITE { 1 } else { -1 };
            }
        }
    }
    res += outpost;
    #[cfg(feature = "display-eval")]
    {
        println!("\nKnights for {}:", if side == WHITE { "White" } else { "Black" });
        println!(
            "\tSupported by pawns: {} -> {}",
            supported_knights_amount,
            KNIGHT_SUPPORTED_BY_PAWN * supported_knights_amount,
        );
        println!("\tOutposts: {} -> {}", _outposts, outpost);
        println!("Sum: {}", res);
    }

    res
}

pub fn piecewise(side: usize, g: &GameState, enemy_defend_by_minors: u64, enemy_defended: u64, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let defended_by_minors = enemy_defend_by_minors;
    let defended_squares = enemy_defended;
    let my_pieces = g.get_pieces_from_side(side);

    let enemy_king_idx = g.get_king_square(1 - side);
    let enemy_king_attackable = KING_ZONE[side][enemy_king_idx] & !defended_by_minors;

    let knight_checks = KNIGHT_ATTACKS[enemy_king_idx];
    let all_pieces = g.get_all_pieces();
    let bishop_checks = PieceType::Bishop.attacks(enemy_king_idx, all_pieces);
    let rook_checks = PieceType::Rook.attacks(enemy_king_idx, all_pieces);
    //Knights
    let mut knight_attackers: i16 = 0;
    let mut knight_attacker_values = EvaluationScore::default();
    let mut mk = EvaluationScore::default();
    let mut knights = g.get_piece(PieceType::Knight, side);
    while knights != 0u64 {
        let idx = knights.trailing_zeros() as usize;
        let targets = PieceType::Knight.attacks(idx, all_pieces) & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mk += KNIGHT_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & knight_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            knight_attackers += 1;
        }
        knight_attacker_values += KNIGHT_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            knight_attacker_values += KNIGHT_SAFE_CHECK;
        }
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_KNIGHT_MOBILITY + mobility] += if side == WHITE { 1 } else { -1 };
            trace.knight_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                trace.knight_safe_check[side] += 1;
            }
        }
        knights ^= square(idx);
    }
    //Bishops
    let mut bishop_attackers: i16 = 0;
    let mut bishop_attacker_values = EvaluationScore::default();
    let (mut mb, mut mb_diag) = (EvaluationScore::default(), EvaluationScore::default());
    let mut bishops = g.get_piece(PieceType::Bishop, side);
    while bishops != 0u64 {
        let idx = bishops.trailing_zeros() as usize;
        let bishop_attack = PieceType::Bishop.attacks(idx, all_pieces ^ square(enemy_king_idx));
        let diagonally_adjacent_pawns = (DIAGONALLY_ADJACENT[idx] & g.get_piece(PieceType::Pawn, side)).count_ones() as usize;
        mb_diag += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS[diagonally_adjacent_pawns];

        let targets = bishop_attack & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mb += BISHOP_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & bishop_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            bishop_attackers += 1;
        }
        bishop_attacker_values += BISHOP_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            bishop_attacker_values += BISHOP_SAFE_CHECK;
        }
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_DIAGONALLY_ADJ_SQ_WPAWNS + diagonally_adjacent_pawns] += if side == WHITE { 1 } else { -1 };
            trace.normal_coeffs[IDX_BISHOP_MOBILITY + mobility] += if side == WHITE { 1 } else { -1 };
            trace.bishop_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                trace.bishop_safe_check[side] += 1;
            }
        }
        bishops ^= square(idx);
    }
    //Rooks
    let mut rook_attackers: i16 = 0;
    let mut rook_attacker_values = EvaluationScore::default();
    let (mut mr, mut rooks_onopen, mut rooks_on_semi_open, mut rooks_onseventh) = (EvaluationScore::default(), 0i16, 0i16, 0i16);
    let mut rooks = g.get_piece(PieceType::Rook, side);
    while rooks != 0u64 {
        let idx = rooks.trailing_zeros() as usize;
        let rook_attack = PieceType::Rook.attacks(idx, all_pieces ^ square(enemy_king_idx));
        if relative_rank(side, idx) == 6 {
            rooks_onseventh += 1;
        }
        if FILES[file_of(idx)] & g.get_piece_bb(PieceType::Pawn) == 0u64 {
            rooks_onopen += 1;
        } else if (FILES[file_of(idx)] & g.get_piece(PieceType::Pawn, 1 - side)).count_ones() == 1 && (FILES[file_of(idx)] & g.get_piece(PieceType::Pawn, side)) == 0u64 {
            rooks_on_semi_open += 1;
        }

        let targets = rook_attack & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mr += ROOK_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & rook_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            rook_attackers += 1;
        }
        rook_attacker_values += ROOK_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            rook_attacker_values += ROOK_SAFE_CHECK;
        }
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_ROOK_MOBILITY + mobility] += if side == WHITE { 1 } else { -1 };
            trace.rook_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                trace.rook_safe_check[side] += 1;
            }
        }
        rooks ^= square(idx);
    }

    //Queens
    let mut queen_attackers: i16 = 0;
    let mut queen_attacker_values = EvaluationScore::default();
    let (mut queens_onopen, mut queens_on_semi_open) = (0i16, 0i16);
    let mut mq = EvaluationScore::default();
    let mut queens = g.get_piece(PieceType::Queen, side);
    while queens != 0u64 {
        let idx = queens.trailing_zeros() as usize;
        let rooklike_attacks = PieceType::Rook.attacks(idx, all_pieces ^ square(enemy_king_idx));
        let bishoplike_attacks = PieceType::Bishop.attacks(idx, all_pieces ^ square(enemy_king_idx));
        let queen_attack = rooklike_attacks | bishoplike_attacks;

        if FILES[file_of(idx)] & g.get_piece_bb(PieceType::Pawn) == 0u64 {
            queens_onopen += 1;
        } else if (FILES[file_of(idx)] & g.get_piece(PieceType::Pawn, 1 - side)).count_ones() == 1 && (FILES[file_of(idx)] & g.get_piece(PieceType::Pawn, side)) == 0u64 {
            queens_on_semi_open += 1;
        }

        let targets = queen_attack & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mq += QUEEN_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & (bishop_checks | rook_checks) & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            queen_attackers += 1;
        }
        queen_attacker_values += QUEEN_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            queen_attacker_values += QUEEN_SAFE_CHECK;
        }

        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_QUEEN_MOBILITY + mobility] += if side == WHITE { 1 } else { -1 };
            trace.queen_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                trace.queen_safe_check[side] += 1;
            }
        }
        queens ^= square(idx);
    }
    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_ROOK_ON_OPEN] += rooks_onopen as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_ROOK_ON_SEMI_OPEN] += rooks_on_semi_open as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_ROOK_ON_SEVENTH] += rooks_onseventh as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_QUEEN_ON_OPEN] += queens_onopen as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_QUEEN_ON_SEMI_OPEN] += queens_on_semi_open as i8 * if side == WHITE { 1 } else { -1 };
    }

    let attack_mg = ((SAFETY_TABLE[(knight_attacker_values.0 + bishop_attacker_values.0 + rook_attacker_values.0 + queen_attacker_values.0).min(99) as usize].0 as isize
        * ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as usize].0 as isize) as f64
        / 100.0) as i16;
    let attack_eg = ((SAFETY_TABLE[(knight_attacker_values.1 + bishop_attacker_values.1 + rook_attacker_values.1 + queen_attacker_values.1).min(99) as usize].1 as isize
        * ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as usize].1 as isize) as f64
        / 100.0) as i16;
    let attack = EvaluationScore(attack_mg, attack_eg);
    #[cfg(feature = "tuning")]
    {
        trace.attackers[side] = (knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as u8;
    }
    #[allow(clippy::let_and_return)]
    let res = mk
        + mb
        + mr
        + mq
        + mb_diag
        + ROOK_ON_OPEN_FILE_BONUS * rooks_onopen
        + ROOK_ON_SEMI_OPEN_FILE_BONUS * rooks_on_semi_open
        + ROOK_ON_SEVENTH * rooks_onseventh
        + QUEEN_ON_OPEN_FILE_BONUS * queens_onopen
        + QUEEN_ON_SEMI_OPEN_FILE_BONUS * queens_on_semi_open
        + attack;

    #[cfg(feature = "display-eval")]
    {
        println!("\nPiecewise for {}:", if side == WHITE { "White" } else { "Black" });
        println!("\tMobility Knight: {}", mk);
        println!("\tMobility Bishop: {}", mb);
        println!("\tBishop Diagonally Adj: {}", mb_diag);
        println!("\tMobility Rook  : {}", mr);
        println!("\tMobility Queen : {}", mq);
        println!("\tRooks on open  : {} -> {}", rooks_onopen, ROOK_ON_OPEN_FILE_BONUS * rooks_onopen,);
        println!("\tRooks on semi-open  : {} -> {}", rooks_on_semi_open, ROOK_ON_SEMI_OPEN_FILE_BONUS * rooks_on_semi_open,);
        println!("\tQueens on open  : {} -> {}", queens_onopen, QUEEN_ON_OPEN_FILE_BONUS * queens_onopen,);
        println!(
            "\tQueens on semi-open  : {} -> {}",
            queens_on_semi_open,
            QUEEN_ON_SEMI_OPEN_FILE_BONUS * queens_on_semi_open,
        );
        println!("\tRooks on seventh: {} -> {}", rooks_onseventh, ROOK_ON_SEVENTH * rooks_onseventh);
        println!("\tKnight Attackers: Num: {} , Val: {}", knight_attackers, knight_attacker_values);
        println!("\tBishop Attackers: Num: {} , Val: {}", bishop_attackers, bishop_attacker_values);
        println!("\tRook Attackers: Num: {} , Val: {}", rook_attackers, rook_attacker_values);
        println!("\tQueen Attackers: Num: {} , Val: {}", queen_attackers, queen_attacker_values);
        println!(
            "\tSum Attackers: (Num: {} , Val: {}",
            knight_attackers + bishop_attackers + rook_attackers + queen_attackers,
            knight_attacker_values + bishop_attacker_values + rook_attacker_values + queen_attacker_values
        );
        println!(
            "\tAttack MG value: {} * {} / 100.0 -> {}",
            SAFETY_TABLE[(knight_attacker_values.0 + bishop_attacker_values.0 + rook_attacker_values.0 + queen_attacker_values.0).min(99) as usize].0,
            ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as usize].0,
            attack_mg
        );
        println!(
            "\tAttack EG value: {} * {} / 100.0 -> {}",
            SAFETY_TABLE[(knight_attacker_values.1 + bishop_attacker_values.1 + rook_attacker_values.1 + queen_attacker_values.1).min(99) as usize].1,
            ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as usize].1,
            attack_eg
        );
        println!("Sum: {}", res);
    }
    res
}

pub fn king(side: usize, g: &GameState, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let mut pawn_shield = SHIELDING_PAWNS[side][g.get_king_square(side)];
    let mut king_front_span = pawn_front_span(g.get_piece(PieceType::King, side), side);
    king_front_span |= west_one(king_front_span) | east_one(king_front_span);
    let file = file_of(g.get_king_square(side));
    if file == 7 {
        king_front_span |= west_one(king_front_span);
    } else if file == 0 {
        king_front_span |= east_one(king_front_span);
    }
    let mut shields_missing = 0;
    let mut shields_on_open_missing = 0;
    if g.get_full_moves() >= 1 {
        while pawn_shield != 0u64 {
            let idx = pawn_shield.trailing_zeros() as usize;
            if g.get_piece(PieceType::Pawn, side) & pawn_shield & FILES[file_of(idx)] == 0u64 {
                shields_missing += 1;
                if g.get_piece(PieceType::Pawn, 1 - side) & FILES[file_of(idx)] & king_front_span == 0u64 {
                    shields_on_open_missing += 1;
                }
            }
            pawn_shield &= !FILES[file_of(idx)];
        }
    }
    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_SHIELDING_PAWN_MISSING + shields_missing] += if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_SHIELDING_PAWN_ONOPEN_MISSING + shields_on_open_missing] += if side == WHITE { 1 } else { -1 };
    }
    #[allow(clippy::let_and_return)]
    let res = SHIELDING_PAWN_MISSING[shields_missing] + SHIELDING_PAWN_MISSING_ON_OPEN_FILE[shields_on_open_missing];

    #[cfg(feature = "display-eval")]
    {
        println!("\nKing for {}:", if side == WHITE { "White" } else { "Black" });
        println!("\tShield pawn missing: {} -> {}", shields_missing, SHIELDING_PAWN_MISSING[shields_missing],);
        println!(
            "\tShield pawn on open file missing: {} -> {}",
            shields_on_open_missing, SHIELDING_PAWN_MISSING_ON_OPEN_FILE[shields_on_open_missing],
        );
        println!("Sum: {}", res);
    }
    res
}

pub fn get_distance(sq: usize, sq2: usize) -> usize {
    (rank_of(sq) as isize - rank_of(sq2) as isize)
        .abs()
        .max((file_of(sq) as isize - file_of(sq2) as isize).abs()) as usize
}

pub fn pawns(side: usize, g: &GameState, defended: u64, enemy_defended: u64, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let mut res = EvaluationScore::default();

    let empty = !g.get_all_pieces();
    let pawns = g.get_piece(PieceType::Pawn, side);
    let enemy_pawns = g.get_piece(PieceType::Pawn, 1 - side);
    //Bitboards
    let pawn_file_fill = file_fill(pawns);
    let front_span = pawn_front_span(pawns, side);
    let mut enemy_front_spans = pawn_front_span(enemy_pawns, side ^ 1);
    enemy_front_spans |= west_one(enemy_front_spans) | east_one(enemy_front_spans);
    let (my_west_attacks, my_east_attacks, enemy_pawn_attacks) = (pawn_west_targets(side, pawns), pawn_east_targets(side, pawns), pawn_targets(1 - side, enemy_pawns));
    let my_pawn_attacks = my_west_attacks | my_east_attacks;
    let (my_pawn_pushes, my_pawn_double_pushes) = (movegen::single_push_pawn_targets(side, pawns, empty), movegen::double_push_pawn_targets(side, pawns, empty));

    let is_attackable = west_one(front_span) | east_one(front_span);
    let enemy_pieces = g.get_pieces_from_side(1 - side);

    let doubled_pawns = (pawns & front_span).count_ones() as i16;
    let isolated_pawns = (pawns & !west_one(pawn_file_fill) & !east_one(pawn_file_fill)).count_ones() as i16;
    let backward_pawns = (forward_one(pawns, side) & enemy_pawn_attacks & !is_attackable & !enemy_pawns).count_ones() as i16;
    let mut supported_pawns = pawns & my_pawn_attacks;
    let _supported_amt = supported_pawns.count_ones() as usize;
    let mut supp = EvaluationScore::default();
    while supported_pawns != 0u64 {
        let index = supported_pawns.trailing_zeros() as usize;
        supported_pawns ^= square(index);
        supp += PAWN_SUPPORTED_VALUE[side][index];
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_PAWN_SUPPORTED + white_pov(index, side)] += if side == WHITE { 1 } else { -1 };
        }
    }
    res += supp;

    let center_attack_pawns = (pawns & pawn_targets(side ^ 1, INNER_CENTER)).count_ones() as i16;
    let pawn_mobility = (my_west_attacks.count_ones() + my_east_attacks.count_ones() + my_pawn_pushes.count_ones() + my_pawn_double_pushes.count_ones()) as i16;
    res += PAWN_DOUBLED_VALUE * doubled_pawns
        + PAWN_ISOLATED_VALUE * isolated_pawns
        + PAWN_BACKWARD_VALUE * backward_pawns
        + PAWN_ATTACK_CENTER * center_attack_pawns
        + PAWN_MOBILITY * pawn_mobility;

    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_PAWN_DOUBLED] += doubled_pawns as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_PAWN_ISOLATED] += isolated_pawns as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_PAWN_BACKWARD] += backward_pawns as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_PAWN_ATTACK_CENTER] += center_attack_pawns as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_PAWN_MOBILITY] += pawn_mobility as i8 * if side == WHITE { 1 } else { -1 };
    }
    //Passers
    let mut passed_pawns: u64 = pawns & !enemy_front_spans;
    let (mut passer_score, mut _passer_normal, mut _passer_notblocked) = (EvaluationScore::default(), 0, 0);
    let mut passer_dist = EvaluationScore::default();
    let mut weak_passers = 0;
    let behind_passers = pawn_front_span(passed_pawns, side ^ 1);
    let rooks_support_passer = (behind_passers & g.get_rook_like_bb(side)).count_ones() as i16;
    let enemy_rooks_attack_passer = (behind_passers & g.get_rook_like_bb(1 - side)).count_ones() as i16;
    res += ROOK_BEHIND_SUPPORT_PASSER * rooks_support_passer + ROOK_BEHIND_ENEMY_PASSER * enemy_rooks_attack_passer;
    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_ROOK_BEHIND_SUPPORT_PASSER] += rooks_support_passer as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_ROOK_BEHIND_ENEMY_PASSER] += enemy_rooks_attack_passer as i8 * if side == WHITE { 1 } else { -1 };
    }
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        //Passed and blocked
        _passer_normal += 1;
        passer_score += PAWN_PASSED_VALUES[relative_rank(side, idx)];
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_PAWN_PASSED + relative_rank(side, idx)] += if side == WHITE { 1 } else { -1 };
        }
        //A weak passer is an attacked and not defended passer
        let weak_passer = square(idx) & enemy_defended != 0u64 && square(idx) & defended == 0u64;
        if weak_passer {
            //Weak passer
            weak_passers += 1;
        }
        //An unblocked passer is a) not weak b) all the squares until conversions are either not attacked or defended and unoccupied or attacked
        if !weak_passer && pawn_front_span(square(idx), side) & (enemy_defended | enemy_pieces) & !defended == 0u64 {
            //Passed and not blocked
            _passer_notblocked += 1;
            passer_score += PAWN_PASSED_NOT_BLOCKED_VALUES[relative_rank(side, idx)];
            #[cfg(feature = "tuning")]
            {
                trace.normal_coeffs[IDX_PAWN_PASSED_NOTBLOCKED + relative_rank(side, idx)] += if side == WHITE { 1 } else { -1 };
            }
        }

        //Distance to kings
        let d_myking = get_distance(idx, g.get_king_square(side));
        let d_enemyking = get_distance(idx, g.get_king_square(1 - side));
        let sub_dist = ((d_myking as isize - d_enemyking as isize) + 6) as usize;
        passer_dist += PASSED_KING_DISTANCE[d_myking - 1] + PASSED_ENEMY_KING_DISTANCE[d_enemyking - 1] + PASSED_SUBTRACT_DISTANCE[sub_dist];
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_PAWN_PASSED_KINGDISTANCE + d_myking - 1] += if side == WHITE { 1 } else { -1 };
            trace.normal_coeffs[IDX_PAWN_PASSED_ENEMYKINGDISTANCE + d_enemyking - 1] += if side == WHITE { 1 } else { -1 };
            trace.normal_coeffs[IDX_PAWN_PASSED_SUBDISTANCE + sub_dist] += if side == WHITE { 1 } else { -1 };
        }
        passed_pawns ^= square(idx);
    }
    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_PAWN_PASSED_WEAK] += weak_passers as i8 * if side == WHITE { 1 } else { -1 };
    }
    res += passer_score + PAWN_PASSED_WEAK * weak_passers + passer_dist;
    #[cfg(feature = "display-eval")]
    {
        println!("\nPawns for {}:", if side == WHITE { "White" } else { "Black" });
        println!("\tDoubled: {} -> {}", doubled_pawns, PAWN_DOUBLED_VALUE * doubled_pawns);
        println!("\tIsolated: {} -> {}", isolated_pawns, PAWN_ISOLATED_VALUE * isolated_pawns,);
        println!("\tBackward: {} -> {}", backward_pawns, PAWN_BACKWARD_VALUE * backward_pawns,);
        println!("\tSupported: {} -> {}", _supported_amt, supp);
        println!("\tAttack Center: {} -> {}", center_attack_pawns, PAWN_ATTACK_CENTER * center_attack_pawns,);
        println!("\tMobility: {} -> {}", pawn_mobility, PAWN_MOBILITY * pawn_mobility,);
        println!("\tPasser Blocked/Not Blocked: {} , {} -> {}", _passer_normal, _passer_notblocked, passer_score);
        println!("\tRook behind passer: {} -> {}", rooks_support_passer, ROOK_BEHIND_SUPPORT_PASSER * rooks_support_passer,);
        println!(
            "\tEnemy Rook behind passer: {} -> {}",
            enemy_rooks_attack_passer,
            ROOK_BEHIND_ENEMY_PASSER * enemy_rooks_attack_passer,
        );
        println!("\tWeak passer: {} -> {}", weak_passers, PAWN_PASSED_WEAK * weak_passers,);
        println!("\tPassers distance to kings -> {}", passer_dist);
        println!("Sum: {}", res);
    }
    res
}

pub fn piece_values(side: usize, g: &GameState, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let mut res = EvaluationScore::default();

    // Piece counts
    let my_pawns = g.get_piece(PieceType::Pawn, side).count_ones() as i16;
    let my_knights = g.get_piece(PieceType::Knight, side).count_ones() as i16;
    let my_bishops = g.get_piece(PieceType::Bishop, side).count_ones() as i16;
    let my_rooks = g.get_piece(PieceType::Rook, side).count_ones() as i16;
    let my_queens = g.get_piece(PieceType::Queen, side).count_ones() as i16;
    let all_pawns = g.get_piece_bb(PieceType::Pawn).count_ones() as usize;

    res += PAWN_PIECE_VALUE * my_pawns
        + (KNIGHT_PIECE_VALUE + KNIGHT_VALUE_WITH_PAWNS[all_pawns]) * my_knights
        + BISHOP_PIECE_VALUE * my_bishops
        + BISHOP_PAIR_BONUS * (my_bishops > 1) as i16
        + ROOK_PIECE_VALUE * my_rooks
        + QUEEN_PIECE_VALUE * my_queens;

    #[cfg(feature = "tuning")]
    {
        trace.normal_coeffs[IDX_PAWN_PIECE_VALUE] += my_pawns as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_KNIGHT_VALUE_WITH_PAWN + all_pawns] += my_knights as i8 * if side == WHITE { 1 } else { -1 };
        trace.knights += my_knights as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_KNIGHT_PIECE_VALUE] += my_knights as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_BISHOP_PIECE_VALUE] += my_bishops as i8 * if side == WHITE { 1 } else { -1 };
        if my_bishops > 1 {
            trace.normal_coeffs[IDX_BISHOP_PAIR] += if side == WHITE { 1 } else { -1 };
        }
        trace.normal_coeffs[IDX_ROOK_PIECE_VALUE] += my_rooks as i8 * if side == WHITE { 1 } else { -1 };
        trace.normal_coeffs[IDX_QUEEN_PIECE_VALUE] += my_queens as i8 * if side == WHITE { 1 } else { -1 };
    }
    #[cfg(feature = "display-eval")]
    {
        println!("\nPiece values for {}", if side == WHITE { "White" } else { "Black" });
        println!("\tPawns: {} -> {}", my_pawns, PAWN_PIECE_VALUE * my_pawns,);
        println!(
            "\tKnights: {} -> {}",
            my_knights,
            (KNIGHT_PIECE_VALUE + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights,
        );
        println!("\tBishops: {} -> {}", my_bishops, BISHOP_PIECE_VALUE * my_bishops,);
        if my_bishops > 1 {
            println!("\tBishop-Pair: {} -> {}", 1, BISHOP_PAIR_BONUS);
        }
        println!("\tRooks: {} -> {}", my_rooks, ROOK_PIECE_VALUE * my_rooks,);
        println!("\tQueens: {} -> {}", my_queens, QUEEN_PIECE_VALUE * my_queens,);
        println!("Sum: {}", res);
    }
    res
}
