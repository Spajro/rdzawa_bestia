use crate::engine::Engine;
use chess::{ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;
use chess::Color::White;
use Color::Black;
use crate::io::uci::Position::{FEN, START};

pub struct UciResult {
    pub msg: Option<String>,
    pub next_color: Color,
}

#[derive(PartialEq)]
pub enum Position {
    FEN(String),
    START,
}

struct ParseResult {
    fen: Position,
    moves: Vec<ChessMove>,
}

pub fn handle_uci(uci: &String, engine: &mut dyn Engine, next_color: Color) -> UciResult {
    let tokens: Vec<&str> = uci.split(' ').collect();
    let mut time: Option<u64> = get_time(&tokens, next_color);
    match tokens[0] {
        "uci" => start(),
        "isready" => is_ready(),
        "ucinewgame" => restart(engine),
        "go" => go(engine, time.unwrap(), next_color),
        "stop" => stop(engine, next_color),
        "position" => update(engine, tokens, next_color),
        "quit" => quit(),
        &_ => UciResult { msg: Some("Unknown command |".to_string() + uci + "|"), next_color: next_color }
    }
}

fn get_time(tokens: &Vec<&str>, next_color: Color) -> Option<u64> {
    for i in (1..tokens.len()).step_by(2) {
        if tokens[i] == "movetime" {
            return Some(tokens[i + 1].parse().unwrap());
        }
        if tokens[i] == "wtime" && next_color == White {
            return Some(tokens[i + 1].parse().unwrap());
        }
        if tokens[i] == "btime" && next_color == Black {
            return Some(tokens[i + 1].parse().unwrap());
        }
    }
    None
}

fn start() -> UciResult {
    UciResult {
        msg: Some("id name rdzawa_bestia\nuciok".parse().unwrap()),
        next_color: White,
    }
}

fn is_ready() -> UciResult {
    UciResult {
        msg: Some("readyok".parse().unwrap()),
        next_color: White,
    }
}

fn restart(engine: &mut dyn Engine) -> UciResult {
    engine.restart();
    UciResult {
        msg: None,
        next_color: White,
    }
}

fn go(engine: &mut dyn Engine, time: u64, next_color: Color) -> UciResult {
    engine.start(time);
    UciResult {
        msg: None,
        next_color: swap_color(next_color),
    }
}

fn stop(engine: &mut dyn Engine, next_color: Color) -> UciResult {
    engine.stop();
    UciResult {
        msg: None,
        next_color: next_color,
    }
}

fn quit() -> UciResult {
    process::exit(0);
}

fn update(engine: &mut dyn Engine, tokens: Vec<&str>, next_color: Color) -> UciResult {
    let parsed = parse_update_tokens(tokens);
    engine.update(parsed.fen, parsed.moves);
    UciResult {
        msg: None,
        next_color: swap_color(next_color),
    }
}

fn parse_update_tokens(tokens: Vec<&str>) -> ParseResult {
    let mut i = tokens.len() - 1;
    let mut moves = Vec::new();
    while i > 1 {
        let mv = ChessMove::from_str(tokens[i]);
        if mv.is_ok() {
            moves.push(mv.unwrap())
        } else {
            break;
        }
        i -= 1;
    }
    let fen_string = tokens[1..=i].join(" ");
    let fen = if fen_string == "startpos" {
        START
    } else {
        FEN(fen_string)
    };
    moves.reverse();
    ParseResult {
        fen,
        moves,
    }
}

pub fn move_to_uci(mv: ChessMove) -> String {
    String::from("bestmove ").add(mv.to_string().as_str())
}

fn swap_color(color: Color) -> Color {
    return match color {
        White => Black,
        Black => White
    };
}
