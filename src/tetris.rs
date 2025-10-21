use crate::tetrimino::{
    Tetrimino, TetriminoGenerator, TetriminoI, TetriminoJ, TetriminoL,
    TetriminoO, TetriminoS, TetriminoT, TetriminoZ,
};

pub struct Tetris {
    pub game_map: Vec<Vec<u8>>,
    pub current_level: u32,
    pub score: u32,
    pub number_of_lines: u32,
    pub current_tetrimino: Option<Tetrimino>,
}

impl Tetris {
    pub fn new() -> Tetris {
        let mut game_map = Vec::new();
        for _ in 0..16 {
            game_map.push(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        }

        Tetris {
            game_map,
            current_level: 1,
            score: 0,
            number_of_lines: 0,
            current_tetrimino: None,
        }
    }

    pub fn create_new_tetrimino(&self) -> Tetrimino {
        static mut PREV: u8 = 7;
        let mut rand_num = rand::random::<u8>() % 7;

        if unsafe { PREV } == rand_num {
            rand_num = rand::random::<u8>() % 7;
        }

        unsafe {PREV = rand_num; }

        match rand_num {
            0 => TetriminoI::new(),
            1 => TetriminoL::new(),
            2 => TetriminoJ::new(),
            3 => TetriminoO::new(),
            4 => TetriminoS::new(),
            5 => TetriminoZ::new(),
            6 => TetriminoT::new(),
            _ => unreachable!(),
        }
    }

    fn check_lines(&mut self) {
        let mut y = 0;

        while y < self.game_map.len() {
            let mut complete = true;
            for &x in &self.game_map[y] {
                if x == 0 {
                    complete = false;
                    break;
                }
            }

            if complete == true {
                self.game_map.remove(y);
                y -= 1;
                // increase the number of self.lines
            }

            y += 1;
        }

        while self.game_map.len() < 16 {
            self.game_map.insert(0, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        }
    }

    pub fn make_permanent(&mut self) {
        let Some(ref mut tetrimino) = self.current_tetrimino else {
            return;
        };

        let mut shift_y = 0;
        while shift_y < tetrimino.states[tetrimino.current_state as usize].len()
            && tetrimino.y + shift_y < self.game_map.len()
        {
            let mut shift_x = 0;
            while shift_x < tetrimino.states[tetrimino.current_state as usize][shift_y].len()
                && (tetrimino.x + shift_x as isize) < self.game_map[tetrimino.y + shift_y].len() as isize
            {
                if tetrimino.states[tetrimino.current_state as usize][shift_y][shift_x] != 0 {
                    let x = tetrimino.x + shift_x as isize;
                    let y = tetrimino.y + shift_y;
                    self.game_map[y][x as usize] = tetrimino.states[tetrimino.current_state as usize][shift_y][shift_x];
                }
                shift_x += 1;
            }
            shift_y += 1;
        }

        self.check_lines();
        self.current_tetrimino = None;
    }
}
