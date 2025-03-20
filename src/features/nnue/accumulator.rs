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
pub struct Accumulator<const M: usize> {
    v: [[f64; M]; 2],
}

impl<const M: usize> Index<usize> for Accumulator<M> {
    type Output = [f64; M];

    fn index(&self, index: Color) -> &Self::Output {
        &self.v[index]
    }
}

impl<const M: usize> IndexMut<usize> for Accumulator<M> {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl<const M: usize> Accumulator<M> {
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

mod test {
    use crate::features::nnue::accumulator::{Accumulator, BLACK, WHITE};
    use crate::features::nnue::network::LinearLayer;

    #[test]
    fn refresh_test() {
        let mut acc = Accumulator::<2>::new();
        let layer = LinearLayer::<4, 4>::new(
            vec![
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0],
            ],
            [100.0, 101.0, 102.0, 103.0],
        );
        let features: Vec<usize> = vec![1, 3];

        acc.refresh(&layer, &features, WHITE);

        assert_eq!(acc.v[WHITE][0], 106.0); //100+2+4
        assert_eq!(acc.v[WHITE][1], 115.0); //101+6+8
        assert_eq!(acc.v[BLACK][1], 0.0);
        assert_eq!(acc.v[BLACK][1], 0.0);
    }

    #[test]
    fn update_test() {
        let mut acc = Accumulator::<2>::new();
        let layer = LinearLayer::<4, 4>::new(
            vec![
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0],
            ],
            [100.0, 101.0, 102.0, 103.0],
        );
        let features: Vec<usize> = vec![1, 3];
        acc.refresh(&layer, &features, WHITE);

        acc.update(&layer, &vec![0], &vec![3], WHITE);

        assert_eq!(acc.v[WHITE][0], 103.0); // 100+1+2
        assert_eq!(acc.v[WHITE][1], 112.0); //101+5+6
        assert_eq!(acc.v[BLACK][1], 0.0);
        assert_eq!(acc.v[BLACK][1], 0.0);
    }
}
