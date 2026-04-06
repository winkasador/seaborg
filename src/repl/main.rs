mod engine;

use std::io::BufRead;

use shakmaty::{Chess, Move, Position, fen::Fen, san::{San, SanError}, uci::UciMove};

use crate::engine::Engine;

#[derive(Debug)]
enum MoveError {
    Invalid,
    Ambiguous,
    Illegal
}

#[derive(Debug)]
enum EngineMoveError {
    GameIsOver,
    Invalid, // If the engine returns a nonsense response
    Illegal // The engine suggested an illegal move
}

fn main() {
    println!("$: Starting engine process");
    let mut engine = Engine::start("./target/debug/engine");
    engine.send("uci");

    let mut id_name: String = "".to_string();
    let mut id_author: String = "".to_string();

    let config = engine.read_until(|l| l == "uciok");
    for l in config {
        let args: Vec<String> = l.split(" ").map(|s| s.to_string()).collect();
        if args.len() >= 3 && args[0] == "id" {
            let value = args[2..args.len()].join(" ");
            match args[1].as_str() {
                "name" => { id_name = value },
                "author" => { id_author = value },
                _ => {}
            }
        }
    }
    
    println!("$: You are playing {id_name} by {id_author}.");
    engine.send("isready");
    engine.read_until(|l| l == "readyok");
    println!("$: Ready to play");

    let mut automove = true;
    let mut position = Chess::default();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut tokens = line.split_whitespace();
        let token = tokens.next();
        match token {
            // Directly input a UCI call
            Some("relay") => {
                let remainder: Vec<&str> = tokens.collect();
                let input = remainder.join(" ");
                engine.send(&input);
            }
            // Tells the engine to automatically make a move after the user submits theirs.
            Some("automove") => {
                let t = tokens.next();
                match t {
                    Some("on") => { 
                        automove = true;
                        println!("$: Engine will now reply to moves automatically")
                    }
                    Some("off") => {
                        automove = false;
                        println!("$: Engine will no longer reply to moves automatically")
                    }
                    None => {
                        let status = if automove {"on"} else {"off"};
                        println!("$: Automove is currently {status}")
                    }
                    _ => {
                        println!("$: Usage: automove (on/off)")
                    }
                }
            }
            Some("new") => {
                position = Chess::default();
                engine.send("ucinewgame");
                println!("$: Position reset")
            }
            Some("fen") => {
                let fen = Fen::from_position(&position, shakmaty::EnPassantMode::Legal);
                println!("$: {fen}")
            }
            // Stop the engine process
            Some("quit") | Some("exit") => {
                break;
            }
            Some("best") => {
                match get_engine_move(&mut engine, &mut position) {
                    Ok(mv) => println!("$: Best move is {}", San::from_move(&position, mv)),
                    Err(EngineMoveError::GameIsOver) => println!("$: There are no moves in this position."),
                    Err(_) => println!("$: Could not determine best move.")
                }
            }
            Some("yourturn") => {
                handle_engine_turn(&mut engine, &mut position);
            }
            None => {}
            _ => {
                let mv = token.unwrap();
                match parse_user_move(mv, &mut position) {
                    Ok(_) => {
                        if automove {
                            handle_engine_turn(&mut engine, &mut position);
                            // No need to print anything because the engine's response is a confirmation
                        } else {
                            println!("$: OK")
                        }
                    }
                    Err(MoveError::Invalid) => println!("$: '{mv}' is not a valid command or move"),
                    Err(MoveError::Ambiguous) => println!("$: {mv} is ambiguous (include the square the piece is moving from)"),
                    Err(MoveError::Illegal) => println!("$: {mv} is not a legal move")
                }
            }
        }
    }

    engine.shutdown();
}

fn parse_user_move(mv: &str, position: &mut Chess) -> Result<(), MoveError> {
    let san: San = match mv.parse() {
        Ok(s) => s,
        Err(_e) => { 
            return Result::Err(MoveError::Invalid);
        }
    };
    let m = match san.to_move(position) {
        Ok(m) => m,
        Err(e) => {
            if e == SanError::AmbiguousSan {
                return Result::Err(MoveError::Ambiguous);
            }
            else {
                return Result::Err(MoveError::Illegal);
            }
        }
    };
    if !position.is_legal(m) {
        return Result::Err(MoveError::Illegal);
    }
    position.play_unchecked(m);
    Result::Ok(())
}

/// Wrapper function that tries to play the engine move and also prints the response or error to the terminal
fn handle_engine_turn(engine: &mut Engine, position: &mut Chess) {
    match play_engine_move(engine, position) {
        Err(EngineMoveError::GameIsOver) => println!("$: No moves are available in this position."),
        Err(EngineMoveError::Invalid) => println!("$: Error! Engine's response was invalid. Is the game over?"),
        Err(EngineMoveError::Illegal) => println!("$: Error! Engine suggested illegal move."),
        Ok(_) => {}
    }
}

/// Asks the engine for the top move in the current position.
fn get_engine_move(engine: &mut Engine, position: &mut Chess) -> Result<Move, EngineMoveError> {
    let fen = Fen::from_position(position, shakmaty::EnPassantMode::Legal);
    let fen_str = fen.to_string();
    
    engine.send(&("position fen ".to_owned() + &fen_str));
    engine.send("go");

    let response = engine.read_until(|l| l.starts_with("bestmove"));
    let tokens: Vec<&str> = response.last().unwrap().split(" ").collect();
    if tokens.len() < 2 {
        return Result::Err(EngineMoveError::GameIsOver);
    }
    let mv = tokens[1];
    if mv == "0000" {
        return Result::Err(EngineMoveError::GameIsOver);
    }

    let uci_move = UciMove::from_ascii(mv.as_bytes())
        .map_err(|_| EngineMoveError::Invalid)?;
    let standard_move = uci_move.to_move(position)
        .map_err(|_| EngineMoveError::Illegal)?;

    return Result::Ok(standard_move)
}

fn play_engine_move(engine: &mut Engine, position: &mut Chess) -> Result<(), EngineMoveError> {
    let mv = get_engine_move(engine, position)?;
    let san = San::from_move(position, mv);
    position.play_unchecked(mv);
    println!("$: {san}" );
    Result::Ok(())
}