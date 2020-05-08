use crate::board_representation::game_state::*;

const MG_LIMIT: i16 = 9100;
const EG_LIMIT: i16 = 2350;

#[derive(Clone)]
pub struct Phase {
    pub phase: f64,
    pub material_score: i16,
}

impl Phase {
    #[inline(always)]
    pub fn update(&mut self) {
        let mut tmp = self.material_score;
        if tmp > MG_LIMIT {
            tmp = MG_LIMIT;
        }
        if tmp < EG_LIMIT {
            tmp = EG_LIMIT;
        }
        self.phase = f64::from(tmp - EG_LIMIT) * 128. / f64::from(MG_LIMIT - EG_LIMIT);
    }
    #[inline(always)]
    pub fn from_pieces(pieces: &[[u64; 2]; 6]) -> Self {
        let material_score = (pieces[PieceType::Queen as usize][WHITE]
            | pieces[PieceType::Queen as usize][BLACK])
            .count_ones() as i16
            * PieceType::Queen.to_phase_score()
            + (pieces[PieceType::Knight as usize][WHITE]
                | pieces[PieceType::Knight as usize][BLACK])
                .count_ones() as i16
                * PieceType::Knight.to_phase_score()
            + (pieces[PieceType::Bishop as usize][WHITE]
                | pieces[PieceType::Bishop as usize][BLACK])
                .count_ones() as i16
                * PieceType::Bishop.to_phase_score()
            + (pieces[PieceType::Rook as usize][WHITE] | pieces[PieceType::Rook as usize][BLACK])
                .count_ones() as i16
                * PieceType::Rook.to_phase_score();
        let mut res = Phase {
            phase: 0.,
            material_score,
        };
        res.update();
        res
    }
    #[inline(always)]
    pub fn delete_piece(&mut self, piece: PieceType) {
        self.material_score -= piece.to_phase_score();
        self.update();
    }

    #[inline(always)]
    pub fn add_piece(&mut self, piece: PieceType) {
        self.material_score += piece.to_phase_score();
        self.update();
    }
}
