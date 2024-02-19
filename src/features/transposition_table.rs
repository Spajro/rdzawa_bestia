use std::collections::HashMap;
use chess::{Board, ChessMove};
use priority_queue::PriorityQueue;
use crate::io::output::send_info;


type PartHash = u32;
type FullHash = u64;

fn high(hash: FullHash) -> PartHash {
    return (hash >> 32) as u32;
}

pub struct TranspositionTable {
    map: HashMap<PartHash, TableEntry>,
    replacement_queue: PriorityQueue<PartHash, u32>,
    max_size: usize,
}

#[derive(Clone)]
pub struct TableEntry {
    pub key: FullHash,
    pub mv: ChessMove,
    pub score: f32,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            map: HashMap::new(),
            replacement_queue: PriorityQueue::new(),
            max_size: 10000,
        }
    }
    pub fn insert(&mut self, pos: &Board, score: f32, mv: ChessMove, halfmoves: u32) {
        send_info("[TT] insert: ".to_string() + &*score.to_string() + " | " + &*mv.to_string());
        if self.max_size <= self.map.len() {
            self.remove()
        }


        let key = Self::get_key(pos);

        let entry = TableEntry {
            key,
            mv,
            score,
        };

        self.map.insert(Self::get_key_part(key), entry);
        self.replacement_queue.push(Self::get_key_part(key), u32::MAX - halfmoves);
    }

    pub fn find(&self, pos: &Board) -> Option<TableEntry> {
        let key = Self::get_key(pos);

        let table_entry = self.map.get(&Self::get_key_part(key));

        if table_entry.is_none() {
            return None;
        }

        let entry = table_entry.unwrap();

        if key != entry.key {
            return None;
        }

        table_entry.map(|e| e.clone())
    }

    fn get_key(pos: &Board) -> FullHash {
        return pos.get_hash();
    }

    fn get_key_part(key: FullHash) -> PartHash {
        return high(key);
    }
    fn remove(&mut self) {
        let first_entry = self.replacement_queue.pop();

        if first_entry.is_none() {
            return;
        }

        let queue_entry = first_entry.unwrap();

        self.map.remove(&queue_entry.0);
    }
}