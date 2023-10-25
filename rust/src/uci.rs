use std::ops::Add;
use std::str::FromStr;
use shakmaty::{CastlingMode, Move};
use shakmaty::uci::Uci;
use crate::engine::Engine;

pub fn handle_uci(uci: &String, engine: &mut dyn Engine) -> Option<String> {
    let tokens: Vec<&str> = uci.split(' ').collect();
    match tokens[0] {
        "uci" => start(),
        "isready" => is_ready(),
        "ucinewgame" => restart(engine),
        "go" => go(engine),
        "stop" => stop(engine),
        "position" => update(engine, tokens),
        &_ => Some("Unknown command".parse().unwrap())
    }
}

fn start() -> Option<String> {
    Some("id name NAME\nuciok".parse().unwrap())
}

fn is_ready() -> Option<String> {
    Some("readyok".parse().unwrap())
}

fn restart(engine: &mut dyn Engine) -> Option<String> {
    engine.restart();
    None
}

fn go(engine: &mut dyn Engine) -> Option<String> {
    engine.start();
    stop(engine)
}

fn stop(engine: &mut dyn Engine) -> Option<String> {
    engine.stop();
    None
}

fn update(engine: &mut dyn Engine, tokens: Vec<&str>) -> Option<String> {
    engine.update(Uci::from_str(tokens[tokens.len() - 1]).unwrap().to_move(&engine.get_status()).unwrap());
    None
}

pub fn move_to_uci(mv: Move) -> String {
    String::from("bestmove ").add(mv.to_uci(CastlingMode::Standard).to_string().as_str())
}