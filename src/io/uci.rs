use crate::engine::Engine;
use chess::{ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;
use chess::Color::White;
use Color::Black;
use crate::io::uci::Position::{FEN, START};

pub struct State {
    pub engine: Box<dyn Engine>,
    pub next_color: Color,
}

pub struct UciResult {
    pub msg: Option<String>,
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

pub fn handle_uci(uci: &String, state: &mut State) -> UciResult {
    let tokens: Vec<&str> = uci.split(' ').collect();
    let mut time: Option<u64> = get_time(&tokens, state.next_color);
    match tokens[0] {
        "uci" => start(),
        "isready" => is_ready(),
        "ucinewgame" => restart(state),
        "go" => go(state, time.unwrap()),
        "stop" => stop(state),
        "position" => update(state, tokens),
        "quit" => quit(),
        &_ => UciResult { msg: Some("Unknown command |".to_string() + uci + "|") }
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
    }
}

fn is_ready() -> UciResult {
    UciResult {
        msg: Some("readyok".parse().unwrap()),
    }
}

fn restart(state: &mut State) -> UciResult {
    state.engine.restart();
    state.next_color = White;
    UciResult {
        msg: None,
    }
}

fn go(state: &mut State, time: u64) -> UciResult {
    state.engine.start(time);
    state.next_color = swap_color(state.next_color);
    UciResult {
        msg: None,
    }
}

fn stop(state: &mut State) -> UciResult {
    state.engine.stop();
    UciResult {
        msg: None,
    }
}

fn quit() -> UciResult {
    process::exit(0);
}

fn update(state: &mut State, tokens: Vec<&str>) -> UciResult {
    let parsed = parse_update_tokens(tokens);
    state.engine.update(parsed.fen, parsed.moves);
    state.next_color = swap_color(state.next_color);
    UciResult {
        msg: None,
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
