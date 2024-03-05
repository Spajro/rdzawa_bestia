use crate::engine::Engine;
use crate::io::output::send_info;
use chess::{ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;
use chess::Color::White;
use Color::Black;
use crate::io::uci::Fen::{FEN, START};

pub struct UciResult {
    pub msg: Option<String>,
    pub next_color: Color,
}

pub enum Fen {
    FEN(Box<str>),
    START,
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
    send_info("DEBUG ".to_string() + tokens[tokens.len() - 1]);
    let position = tokens[1];
    if tokens.len() == 2 && position == "startpos" {
        return UciResult {
            msg: None,
            next_color: swap_color(next_color),
        };
    }

    let moves = tokens.iter().skip(2).map(|s| ChessMove::from_str(s).unwrap()).collect();
    let fen = match position {
        "startpos" => START,
        _ => FEN(Box::from(position))
    };
    engine.update(fen, moves);
    UciResult {
        msg: None,
        next_color: swap_color(next_color),
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
