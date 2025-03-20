use std::ops::{Index, IndexMut};
use features::nnue::network;
use network::LinearLayer;
use crate::features;

type Color = usize;

const WHITE: Color = 1;
pub const BLACK: Color = 0;

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
pub struct Accumulator<const M:usize> {
    v: [[f64; M]; 2],
}

impl<const M:usize> Index<usize> for Accumulator<M> {
    type Output = [f64; M];

    fn index(&self, index: Color) -> &Self::Output {
        &self.v[index]
    }
}

impl<const M:usize> IndexMut<usize> for Accumulator<M> {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl<const M:usize> Accumulator<M> {
    pub fn new() -> Self {
        Accumulator {
            v: [[0.0; M]; 2],
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

        for &feature in active_features {
            for i in 0..M {
                self[perspective][i] += layer.weight[i][feature];
            }
        }
    }

    pub fn update<const IN: usize, const OUT: usize>(&mut self,
                                                     layer: &LinearLayer<IN, OUT>,
                                                     active_features: &Vec<usize>,
                                                     removed_features: &Vec<usize>,
                                                     perspective: Color,
    ) {
        for &feature in removed_features {
            for i in 0..M {
                self[perspective][i] -= layer.weight[i][feature];
            }
        }
        for &feature in active_features {
            for i in 0..M {
                self[perspective][i] += layer.weight[i][feature];
            }
        }
    }
}
