use crate::NUM_COLS;
use crate::NUM_ROWS;

pub type Frame = Vec<Vec<& 'static str>>;

// Cria um novo frame preenchendo os vetores de colunas e linhas
pub fn new_frame() -> Frame {
    let mut cols = Vec::with_capacity(NUM_COLS);
    for _ in 0..NUM_COLS {
        let mut col = Vec::with_capacity(NUM_ROWS);
        for _ in 0..NUM_ROWS {
            col.push(" ");
        }
        cols.push(col);
    }

    return cols;
}

// Trait que define um 'desenhável' que deve implementar a função 'draw'
pub trait Drawable {
    fn draw(&self, frame:&mut Frame);
}