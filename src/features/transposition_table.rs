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
pub enum EntryType {
    EXACT,
    LOWER,
    UPPER,
}

#[derive(Clone)]
pub struct TableEntry {
    pub key: FullHash,
    pub mv: Option<ChessMove>,
    pub score: i32,
    pub depth: usize,
    pub entry_type: EntryType,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            map: HashMap::new(),
            keys: Vec::new(),
            max_size: 10_usize.pow(7),
            rand: rand::thread_rng(),
        }
    }
    pub fn insert(&mut self, pos: &Board, score: i32, mv: Option<ChessMove>, depth: usize, entry_type: EntryType) {
        if self.max_size <= self.map.len() {
            self.remove();
        }

        let key = Self::get_key(pos);
        self.map.insert(Self::get_key_part(key), TableEntry { key, mv, score, depth, entry_type });
        self.keys.push(Self::get_key_part(key));
    }

    pub fn find(&self, pos: &Board) -> Option<TableEntry> {
        let key = Self::get_key(pos);
        self.map.get(&Self::get_key_part(key))
            .filter(|e| e.key != key)
            .map(|e| e.clone())
    }

    pub fn restart(&mut self) {
        self.map.clear();
        self.keys.clear();
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