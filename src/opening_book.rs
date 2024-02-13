use std::fs;
use std::path::Path;
use std::str::FromStr;
use json::JsonValue;
use shakmaty::{Chess, Move};
use shakmaty::uci::Uci;
use crate::output::send_info;

#[derive(Clone)]
pub struct OpeningBook {
    node: Option<JsonValue>,
}

pub struct QueryResult {
    pub mv: Option<Move>,
    pub book: OpeningBook,
}

impl OpeningBook {
    pub fn new() -> OpeningBook {
        return if Path::new("book.json").exists() {
            let json = fs::read_to_string("book.json").unwrap();
            let book = json::parse(&*json).unwrap();
            OpeningBook {
                node: Some(book)
            }
        } else {
            OpeningBook {
                node: None,
            }
        };
    }

    pub fn update(self, mv: String) -> OpeningBook {
        return if self.node.clone().is_some() && self.node.clone().unwrap().has_key(mv.as_str()) {
            send_info("Move in book:".to_string() + &*mv);
            let nxt = self.node.unwrap()[mv.as_str()].clone();
            OpeningBook {
                node: Some(nxt)
            }
        } else {
            send_info("Move not in book:".to_string() + &*mv);
            OpeningBook {
                node: None
            }
        };
    }

    pub fn try_get_best(self, pos: &Chess) -> QueryResult {
        if self.node.is_none() {
            send_info("No move found".to_string());
            return QueryResult {
                mv: None,
                book: OpeningBook {
                    node: None
                },
            }
        }
        let node = self.node.unwrap();
        if !node.has_key("best") {
            send_info("No move found".to_string());
            return QueryResult {
                mv: None,
                book: OpeningBook {
                    node: None
                },
            }
        }
        let mv = node["best"].as_str().unwrap();
        let nxt = node[mv].clone();
        send_info("Move from book:".to_string() + mv);
        let mov = Uci::from_str(mv).unwrap().to_move(pos).unwrap();
        QueryResult {
            mv: Some(mov),
            book: OpeningBook {
                node: Some(nxt),
            },
        }
    }
}
