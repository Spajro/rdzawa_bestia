use crate::engine::Engine;
use crate::io::output::send_info;
use chess::{ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;
use chess::Color::White;
use Color::Black;

pub struct UciResult {
    pub msg: Option<String>,
    pub next_color: Color,
}

pub fn handle_uci(uci: &String, engine: &mut dyn Engine, next_color: Color) -> UciResult {
    let tokens: Vec<&str> = uci.split(' ').collect();
    let mut time: Option<u64> = None;
    for i in (1..tokens.len()).step_by(2) {
        if tokens[i] == "movetime" {
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
        if tokens[i] == "wtime" && next_color == White {*
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
        if tokens[i] == "btime" && next_color == Black {
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
    }
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
    if tokens.len() == 2 && tokens[1] == "startpos" {
        return UciResult {
            msg: None,
            next_color: swap_color(next_color),
        };
    }
    engine.update(ChessMove::from_str(tokens[tokens.len() - 1]).unwrap());
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
