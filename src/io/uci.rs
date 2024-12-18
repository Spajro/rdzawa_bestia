use crate::engine::Engine;
use chess::{Board, ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;
use chess::Color::White;
use Color::Black;
use crate::io::options::Options;
use crate::io::uci::Position::{FEN, START};
use crate::minmax_engine::MinMaxEngine;

pub struct State {
    pub engine: Box<dyn Engine>,
    pub options: Options,
    pub next_color: Color,
    pub is_set_up: bool,
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
        "isready" => is_ready(state),
        "ucinewgame" => restart(state),
        "go" => go(state, time.unwrap()),
        "stop" => stop(state),
        "position" => update(state, tokens),
        "setoption" => set_option(state, tokens),
        "eval" => evaluate(state),
        "quit" => quit(),
        &_ => UciResult::with("Unknown command |".to_string() + uci + "|")
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
    UciResult::with("id name rdzawa_bestia\nuciok".to_string())
}

fn is_ready(state: &mut State) -> UciResult {
    if !state.is_set_up {
        state.engine = Box::new(MinMaxEngine::new(Board::default(), &state.options));
        state.is_set_up = true;
    }
    UciResult::with("readyok".to_string())
}

fn restart(state: &mut State) -> UciResult {
    state.engine.restart();
    state.next_color = White;
    UciResult::empty()
}

fn go(state: &mut State, time: u64) -> UciResult {
    state.engine.start(time);
    state.next_color = swap_color(state.next_color);
    UciResult::empty()
}

fn stop(state: &mut State) -> UciResult {
    state.engine.stop();
    UciResult::empty()
}

fn quit() -> UciResult {
    process::exit(0);
}

fn update(state: &mut State, tokens: Vec<&str>) -> UciResult {
    let parsed = parse_update_tokens(tokens);
    state.engine.update(parsed.fen, parsed.moves);
    state.next_color = swap_color(state.next_color);
    UciResult::empty()
}

fn set_option(state: &mut State, tokens: Vec<&str>) -> UciResult {
    if tokens.len() > 2 {
        if tokens[1] == "name" {
            let key = tokens[2];
            if tokens.len() > 4 {
                if tokens[3] == "value" {
                    let value = tokens[4];
                    state.options.add_value(key.to_string(), value.to_string());
                }
            } else {
                state.options.add_flag(key.to_string());
            }
        }
    }
    UciResult::empty()
}

fn evaluate(state: &State) -> UciResult {
    UciResult::with("eval ".to_string() + &*state.engine.evaluate().to_string())
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

impl UciResult {
    pub fn empty() -> Self {
        Self {
            msg: None,
        }
    }

    pub fn with(msg: String) -> Self {
        Self {
            msg: Some(msg),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            engine: Box::new(MinMaxEngine::new(Board::default(), &Options::new())),
            options: Options::new(),
            next_color: White,
            is_set_up: false,
        }
    }
}