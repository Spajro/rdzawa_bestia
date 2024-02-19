use crate::engine::Engine;
use crate::io::output::send_info;
use chess::{ChessMove, Color};
use std::ops::Add;
use std::process;
use std::str::FromStr;

pub fn handle_uci(uci: &String, engine: &mut dyn Engine) -> Option<String> {
    let tokens: Vec<&str> = uci.split(' ').collect();
    let mut time: Option<u64> = None;
    for i in (1..tokens.len()).step_by(2) {
        if tokens[i] == "movetime" {
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
        if tokens[i] == "wtime" && engine.get_status().side_to_move() == Color::White {
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
        if tokens[i] == "btime" && engine.get_status().side_to_move() == Color::Black {
            time = Some(tokens[i + 1].parse().unwrap());
            break;
        }
    }
    match tokens[0] {
        "uci" => start(),
        "isready" => is_ready(),
        "ucinewgame" => restart(engine),
        "go" => go(engine, time.unwrap()),
        "stop" => stop(engine),
        "position" => update(engine, tokens),
        "quit" => quit(),
        &_ => Some("Unknown command |".to_string() + uci + "|"),
    }
}

fn start() -> Option<String> {
    Some("id name rdzawa_bestia\nuciok".parse().unwrap())
}

fn is_ready() -> Option<String> {
    Some("readyok".parse().unwrap())
}

fn restart(engine: &mut dyn Engine) -> Option<String> {
    engine.restart();
    None
}

fn go(engine: &mut dyn Engine, time: u64) -> Option<String> {
    engine.start(time);
    None
}

fn stop(engine: &mut dyn Engine) -> Option<String> {
    engine.stop();
    None
}

fn quit() -> Option<String> {
    process::exit(0);
}

fn update(engine: &mut dyn Engine, tokens: Vec<&str>) -> Option<String> {
    send_info("DEBUG ".to_string() + tokens[tokens.len() - 1]);
    if tokens.len() == 2 && tokens[1] == "startpos" {
        return None;
    }
    engine.update(ChessMove::from_str(tokens[tokens.len() - 1]).unwrap());
    None
}

pub fn move_to_uci(mv: ChessMove) -> String {
    String::from("bestmove ").add(mv.to_string().as_str())
}
