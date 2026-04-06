mod search;
mod eval;

use std::io::BufRead;
use std::str::SplitWhitespace;

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, FromSetup, Position};

use search::search;
use eval::evaluate;

fn main() {
    let stdin = std::io::stdin();

    let mut position = Chess::default();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens = line.split_whitespace();

        process_uci(tokens, &mut position);
    }
}

fn process_uci(mut tokens: SplitWhitespace, position: &mut Chess) -> bool {
    match tokens.next() {
        Some("uci") => {
            println!("id name Seaborg");
            println!("id author Winkassador!");
            println!("uciok");
        }
        Some("isready") => println!("readyok"),
        Some("ucinewgame") => { *position = Chess::default() }
        Some("position") => build_position(position, tokens),
        Some("go") => {
            let result = search(position, 4);
            let best = result.0;
            let evaluation = result.1;
            match best {
                Some(best) => println!("bestmove {} {evaluation}", best.to_uci(shakmaty::CastlingMode::Standard)),
                None => println!("bestmove 0000")
            }
        }
        Some("quit") => {
            return true;
        }
        // DEBUG COMMANDS
        // Evaluates the score of a specific position.
        Some("eval") => {
            let mv = tokens.next();
            if mv.is_some() {
                let ascii_move = mv.unwrap();
                let uci_move = UciMove::from_ascii(ascii_move.as_bytes()).unwrap();
                let standard_move = uci_move.to_move(position).unwrap();
                let mut next_pos = position.clone();
                next_pos.play_unchecked(standard_move);
                let evaluation = evaluate(&next_pos);
                println!("evaluation {}", evaluation)
            }
        }
        _ => {}
    }

    false
}

fn build_position(position: &mut Chess, mut tokens: SplitWhitespace) {
    match tokens.next() {
        Some("startpos") => *position = Chess::default(),
        Some("fen") => {
            let fen = tokens.by_ref().take(6).collect::<Vec<_>>().join(" ");
            *position = Chess::from_setup(Fen::from_ascii(fen.as_bytes()).unwrap().into_setup(), shakmaty::CastlingMode::Standard).unwrap();
        }
        _ => {}
    }

    if tokens.next() == Some("moves") {
        for mv in tokens {
            let uci_move = UciMove::from_ascii(mv.as_bytes()).unwrap();
            let standard_move = uci_move.to_move(&*position).unwrap();
            position.play_unchecked(standard_move);
        }
    }
}