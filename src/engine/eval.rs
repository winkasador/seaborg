use shakmaty::{Bitboard, Board, Chess, Color, File, Position};

const DOUBLED_PAWN_PENALTY: i32 = 45;  // penalty for doubled pawns
const MOBILITY_BONUS: i32 = 0; // bonus for each legal move

/// Evaluates how strong the side to move is in this position, in centipawns.
pub fn evaluate(position: &Chess) -> i32 {
    let mut score: i32 = 0;

    let side = position.turn();
    let board = position.board();

    score += material_advantage(side, &board);

    let my_doubled_pawns = doubled_pawn_count(side, board);
    let their_doubled_pawns = doubled_pawn_count(side.other(), board);

    let moves = position.legal_moves().len() as i32;
    score += moves * MOBILITY_BONUS;

    score -= my_doubled_pawns * DOUBLED_PAWN_PENALTY;
    score += their_doubled_pawns * DOUBLED_PAWN_PENALTY;

    score
}

/// Counts up the amount of material a side has in centipawns.
fn count_material(side: Color, board: &Board) -> i32 {
    let mat = board.material_side(side);
    mat.pawn as i32 * 100 + mat.bishop as i32 * 300 + mat.knight as i32 * 300 + mat.rook as i32 * 500 + mat.queen as i32 * 900
}

fn material_advantage(side: Color, board: &Board) -> i32 {
    let w_mat = count_material(Color::White, board) as i32;
    let b_mat = count_material(Color::Black, board) as i32;

    if side == Color::White {
        return w_mat - b_mat;
    }

    b_mat - w_mat
}

/// Returns the number of files with more than one of the specified side's pawns.
fn doubled_pawn_count(side: Color, board: &Board) -> i32 {
    let pawns = board.pawns();
    let side_bitboard = board.by_color(side);
    
    let side_pawns = pawns & side_bitboard;

    let mut count = 0;

    for file in File::ALL {
        let file_bitboard = Bitboard::from_file(file);
        let check = side_pawns & file_bitboard;
        if check.more_than_one() {
            count += 1;
        }
    }

    count
}