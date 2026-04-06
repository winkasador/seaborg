use shakmaty::{Chess, Move, Position};

use crate::eval::evaluate;

const NEGATIVE_INF: i32 = -10_000_000;
const POSITIVE_INF: i32 = 10_000_000;

pub fn search(root: &Chess, depth: u32) -> (Option<Move>, i32) {
    let mut best_move: Option<Move> = None;
    let mut top_score: i32 = NEGATIVE_INF;

    let moves = root.legal_moves();
    for mv in moves {
        let mut position = root.clone();
        position.play_unchecked(mv);

        let score = minimax(&position, depth - 1, false);
        if score > top_score {
            best_move = Some(mv);
            top_score = score;
        }
    }

    (best_move, top_score)
}

fn minimax(position: &Chess, depth: u32, is_maximising: bool) -> i32 {
    if depth == 0 {
        return evaluate(position);
    }

    if is_maximising {
        let mut best = NEGATIVE_INF;

        for mv in position.legal_moves() {
            let mut child = position.clone();
            child.play_unchecked(mv);
            let score = minimax(&child, depth - 1, false);
            best = score.max(best);
        }
        
        return best;
    }
    else {
        let mut best = POSITIVE_INF;
        for mv in position.legal_moves() {
            let mut child = position.clone();
            child.play_unchecked(mv);
            let score = minimax(&child, depth - 1, true);
            best = score.min(best);
        }

        return best;
    }
}