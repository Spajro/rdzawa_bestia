use crate::io::output::send_info;
use chess::ChessMove;
use json::JsonValue;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone)]
pub struct OpeningBook {
    node: Option<JsonValue>,
    path: String,
}

pub struct QueryResult {
    pub mv: Option<ChessMove>,
    pub book: OpeningBook,
}

impl OpeningBook {
    pub fn new(path: &String) -> Self {
        return if Path::new(path).exists() {
            let json = fs::read_to_string(path).unwrap();
            let book = json::parse(&*json).unwrap();
            OpeningBook { node: Some(book), path: path.clone() }
        } else {
            OpeningBook { node: None, path: path.clone() }
        };
    }

    pub fn empty() -> Self {
        return OpeningBook {
            node: None,
            path: "".to_string(),
        };
    }

    pub fn update(self, mv: String) -> Self {
        if self.node.is_none() {
            send_info("Book empty: ".to_string() + &*mv);
            return OpeningBook { node: None, path: self.path };
        }

        let node = self.node.unwrap();
        if !node.has_key(mv.as_str()) {
            send_info("Move not in book: ".to_string() + &*mv);
            return OpeningBook { node: None, path: self.path };
        };

        send_info("Move in book:".to_string() + &*mv);
        let nxt = node[mv.as_str()].clone();
        OpeningBook { node: Some(nxt), path: self.path }
    }

    pub fn try_get_best(self) -> QueryResult {
        if self.node.is_none() {
            send_info("No move found".to_string());
            return QueryResult {
                mv: None,
                book: OpeningBook { node: None, path: self.path },
            };
        }

        let node = self.node.unwrap();
        if !node.has_key("best") {
            send_info("No move found".to_string());
            return QueryResult {
                mv: None,
                book: OpeningBook { node: None, path: self.path },
            };
        }

        let mv = node["best"].as_str().unwrap();
        let nxt = node[mv].clone();
        send_info("Move from book: ".to_string() + mv);
        let mov = ChessMove::from_str(mv).unwrap();
        QueryResult {
            mv: Some(mov),
            book: OpeningBook { node: Some(nxt), path: self.path },
        }
    }

    pub fn restart(&self) -> Self {
        Self::new(&self.path)
    }
}
