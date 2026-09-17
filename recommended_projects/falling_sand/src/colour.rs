#[derive(Debug, Clone, Copy)]
pub struct ColourState {
    pub amplitudes: [f64; 3],
}

impl ColourState {
   pub fn red() -> Self {
        Self {
            amplitudes: [1.0, 0.0, 0.0],
        }
    }

    pub fn green() -> Self {
        Self {
            amplitudes: [0.0, 1.0, 0.0],
        }
    }

    pub fn blue() -> Self {
        Self {
            amplitudes: [0.0, 0.0, 1.0],
        }
    }

   pub fn norm_squared(&self) -> f64 {
        self.amplitudes
            .iter()
            .map(|a| a * a)
            .sum()
    }
}


pub enum Generator {
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
}


#[derive(Debug, Clone, Copy)]
pub struct ColourOperator {
    pub matrix: [[f64; 3]; 3],
}

impl ColourOperator {
    
    pub fn new(generator: Generator) -> Self {
        let amplitudes = match generator {
            Generator::T1 => 
                [
                    [0.0, 0.5, 0.0],
                    [0.5, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                ], 
            Generator::T3 =>
                [
                    [0.5, 0.0, 0.0],
                    [0.0, -0.5, 0.0],
                    [0.0, 0.0, 0.0],
                ], 
            Generator::T4 =>
                [
                    [0.0, 0.0, 0.5],
                    [0.0, 0.0, 0.0],
                    [0.5, 0.0, 0.0],
                ], 
            _ => todo!(),
        };

        Self {
            matrix: amplitudes,
        }


    }


    pub fn apply(&self, state: ColourState) -> ColourState {
        let mut result = [0.0; 3];

        for row in 0..3 {
            for col in 0..3 {
                result[row] += self.matrix[row][col]
                    * state.amplitudes[col];
            }
        }

        ColourState {
            amplitudes: result,
        }

    }
}














