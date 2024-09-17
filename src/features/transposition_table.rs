use std::collections::HashMap;
use chess::{Board, ChessMove};
use rand::Rng;
use rand::rngs::ThreadRng;


type PartHash = u32;
type FullHash = u64;

fn high(hash: FullHash) -> PartHash {
    (hash >> 32) as u32
}

fn low(hash: FullHash) -> PartHash {
    hash as PartHash
}

pub struct TranspositionTable {
    map: HashMap<PartHash, TableEntry>,
    keys: Vec<PartHash>,
    max_size: usize,
    rand: ThreadRng,
}

#[derive(Clone)]
pub struct TableEntry {
    pub key: FullHash,
    pub mv: ChessMove,
    pub score: f32,
    pub depth: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            map: HashMap::new(),
            keys: Vec::new(),
            max_size: 10000,
            rand: rand::thread_rng(),
        }
    }
    pub fn insert(&mut self, pos: &Board, score: f32, mv: ChessMove, depth: usize) {
        //send_info("[TT] insert: ".to_string() + &*score.to_string() + " | " + &*mv.to_string());
        if self.max_size <= self.map.len() {
            self.remove();
        }


        let key = Self::get_key(pos);

        let entry = TableEntry {
            key,
            mv,
            score,
            depth,
        };

        self.map.insert(Self::get_key_part(key), entry);
        self.keys.push(Self::get_key_part(key));
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
        let index = self.rand.gen_range(0..self.max_size);
        let hash_to_remove = self.keys.swap_remove(index);
        self.map.remove(&hash_to_remove);
    }
}