use std::ops::{Index, IndexMut};
use features::nnue::network;
use network::LinearLayer;
use crate::features;


pub const M: usize = 128;

type Color = usize;

const WHITE: Color = 1;
const BLACK: Color = 0;

pub fn not(color: Color) -> Color {
    if color == WHITE {
        BLACK
    } else {
        WHITE
    }
}

pub fn from(color: chess::Color) -> Color {
    match color {
        chess::Color::White => WHITE,
        chess::Color::Black => BLACK,
    }
}

#[derive(Clone)]
pub struct Accumulator {
    v: [[f32; M]; 2],
}

impl Index<usize> for Accumulator {
    type Output = [f32; M];

    fn index(&self, index: Color) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Accumulator {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl Accumulator {
    pub fn new() -> Self {
        Accumulator {
            v: [[0.0; 128]; 2],
        }
    }
    pub fn refresh<const IN: usize, const OUT: usize>(
        &mut self,
        layer: &LinearLayer<IN, OUT>,
        active_features: &Vec<usize>,
        perspective: Color,
    ) {
        for i in 0..M {
            self[perspective][i] = layer.bias[i];
        }
        for feature in active_features {
            for i in 0..M {
                self[perspective][i] += layer.weight[i][*feature];
            }
        }
    }

    pub fn update<const IN: usize, const OUT: usize>(&mut self,
                                                     layer: &LinearLayer<IN, OUT>,
                                                     active_features: &Vec<usize>,
                                                     removed_features: &Vec<usize>,
                                                     perspective: Color,
    ) {
        for feature in removed_features {
            for i in 1..M {
                self[perspective][i] -= layer.weight[i][*feature];
            }
        }
        for feature in active_features {
            for i in 1..M {
                self[perspective][i] += layer.weight[i][*feature];
            }
        }
    }
}
