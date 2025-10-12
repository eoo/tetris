extern crate sdl3;
extern crate rand;

use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::fs::File;
use std::io::{self, Read, Write};

use sdl3::libc::tm;
use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::render::{Canvas, Texture, TextureCreator};
use sdl3::video::{Window, WindowContext};
use sdl3::image::LoadTexture;

const TEXTURE_SIZE: u32 = 32;

#[derive(Clone, Copy)]
enum TextureColor {
    Green,
    Blue
}

fn create_texture_rect<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    color: TextureColor,
    size: u32
) -> Option<Texture<'a>> {

    if let Ok(mut square_texture) = texture_creator.create_texture_target(None, size, size) {
        canvas.with_texture_canvas(&mut square_texture, |texture| {
            match color {
                TextureColor::Green => texture.set_draw_color(Color::RGB(0, 255, 0)),
                TextureColor::Blue => texture.set_draw_color(Color::RGB(0, 0, 255)),
            }
            texture.clear();
        }).expect("Failed to color a texture");
        Some(square_texture)
    } else {
        None
    }
}

fn write_into_file(content: &str, file_name: &str) -> io::Result<()> {
    let mut f = File::create(file_name)?;
    f.write_all(content.as_bytes())
}

fn read_from_file(file_name: &str) -> io::Result<String> {
    let mut f = File::open(file_name)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    Ok(content)
}

fn slice_to_string(slice: &[u32]) -> String {
    slice.iter().map(|highscore| highscore.to_string()).collect::<Vec<_>>().join(" ")
}

fn save_highscore_and_lines(highscores: &[u32], number_of_lines: &[u32]) -> bool {
    let s_highscores = slice_to_string(highscores);
    let s_number_of_lines = slice_to_string(number_of_lines);

    write_into_file(&format!("{}\n{}\n", s_highscores, s_number_of_lines), "highscores.txt").is_ok()
}

fn line_to_slice(line: &str) -> Vec<u32> {
    line.split(" ")
        .filter_map(|num| num.parse::<u32>().ok())
        .collect()
}

fn load_highscores_and_lines() -> Option<(Vec<u32>, Vec<u32>)> {
    if let Ok(content) = read_from_file("highscores.txt") {
        let mut lines = 
            content.splitn(2, "\n")
            .map(|line| line_to_slice(line)).collect::<Vec<_>>();

        if lines.len() == 2 {
            let number_lines = lines.pop().unwrap();
            let highscores = lines.pop().unwrap();
            Some((highscores, number_lines))
        } else {
            None
        }
    } else {
        None
    }
}


type Piece = Vec<Vec<u8>>;
type States = Vec<Piece>;

struct Tetrimino {
    states: States,
    x: isize,
    y: usize,
    current_state: u8,
}

trait TetriminoGenerator {
    fn new() -> Tetrimino;
}

struct TetriminoI;
impl TetriminoGenerator for TetriminoI {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![1, 1, 1, 1],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0],
                        ],
                        vec![
                            vec![0, 1, 0, 0],
                            vec![0, 1, 0, 0],
                            vec![0, 1, 0, 0],
                            vec![0, 1, 0, 0],
                        ]
                    ],
            x: 4,
            y: 0,
            current_state: 0,
        }
    }
}

struct TetriminoL;
impl TetriminoGenerator for TetriminoL {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![2, 2, 2, 0],
                            vec![2, 0, 0, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![2, 2, 0, 0],
                            vec![0, 2, 0, 0],
                            vec![0, 2, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 0, 2, 0],
                            vec![2, 2, 2, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![2, 0, 0, 0],
                            vec![2, 0, 0, 0],
                            vec![2, 2, 0, 0],
                            vec![0, 0, 0, 0]
                        ]
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

struct TetriminoJ;
impl TetriminoGenerator for TetriminoJ {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![3, 3, 3, 0],
                            vec![0, 0, 3, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 3, 0, 0],
                            vec![0, 3, 0, 0],
                            vec![3, 3, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![3, 0, 0, 0],
                            vec![3, 3, 3, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![3, 3, 0, 0],
                            vec![3, 0, 0, 0],
                            vec![3, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ]
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

struct TetriminoO;
impl TetriminoGenerator for TetriminoO {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![4, 4, 0, 0],
                            vec![4, 4, 0, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

struct TetriminoS;
impl TetriminoGenerator for TetriminoS {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![0, 5, 5, 0],
                            vec![5, 5, 0, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 5, 0, 0],
                            vec![0, 5, 5, 0],
                            vec![0, 0, 5, 0],
                            vec![0, 0, 0, 0]
                        ],
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

struct TetriminoZ;
impl TetriminoGenerator for TetriminoZ {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![6, 6, 0, 0],
                            vec![0, 6, 6, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 0, 6, 0],
                            vec![0, 6, 6, 0],
                            vec![0, 6, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

struct TetriminoT;
impl TetriminoGenerator for TetriminoT {
    fn new() -> Tetrimino {
        Tetrimino {
            states: vec![
                        vec![
                            vec![7, 7, 7, 0],
                            vec![0, 7, 0, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 7, 0, 0],
                            vec![7, 7, 0, 0],
                            vec![0, 7, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 7, 0, 0],
                            vec![7, 7, 7, 0],
                            vec![0, 0, 0, 0],
                            vec![0, 0, 0, 0]
                        ],
                        vec![
                            vec![0, 7, 0, 0],
                            vec![0, 7, 7, 0],
                            vec![0, 7, 0, 0],
                            vec![0, 0, 0, 0]
                        ]
                    ],
                x: 4,
                y: 0,
                current_state: 0,
        }
    }
}

fn create_new_tetrimino() -> Tetrimino {
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

impl Tetrimino {
    fn rotate(&mut self, game_map: &[Vec<u8>]) -> bool{
        let mut tmp_state = self.current_state + 1;
        if tmp_state as usize >= self.states.len() {
            tmp_state = 0;
        }

        let x_pos = [0, -1, 1, -2, 2, -3];
        for &x in x_pos.iter() {
            if self.test_position(game_map, tmp_state as usize, self.x + x, self.y) {
                self.current_state = tmp_state;
                self.x += x;
                return true
            }
        }
        false
    }

    fn test_position(
        &self,
        game_map: &[Vec<u8>],
        tmp_state: usize,
        x: isize,
        y: usize,
    ) -> bool {
        for decal_y in 0..4 {
            for decal_x in 0..4 {
                let x = x + decal_x;
                let y = y + decal_y;

                if self.states[tmp_state][decal_y][decal_x as usize] != 0 
                    && (
                        y >= game_map.len() ||
                        x < 0 ||
                        x as usize >= game_map[y].len() ||
                        game_map[y][x as usize] != 0
                    ) {
                        return false;
                }
            }
        }
        true
    }
}


pub fn main() {
    let sdl_context = sdl3::init().expect("SDL initialization failed");
    let video_subsystem = sdl_context.video().expect("Could not get SDL video subsystem");

    let window = video_subsystem.window("Tetris", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .expect("Could not create window");
    
    let mut canvas = window.into_canvas();
    
    let texture_creator = canvas.texture_creator();
    
    let green_square = create_texture_rect(&mut canvas, &texture_creator, TextureColor::Green, TEXTURE_SIZE)
        .expect("Could not create green square texture");

    let blue_square = create_texture_rect(&mut canvas, &texture_creator, TextureColor::Blue, TEXTURE_SIZE)
        .expect("Could not create blue square texture");

    let image_texture = texture_creator.load_texture("assets/space_nebula.png")
        .expect("Could not load image texture");

    let mut event_pump = sdl_context.event_pump().expect("Could not get SDL event pump");
    let timer = SystemTime::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        
        canvas.set_draw_color(Color::RGB(216, 0, 0));
        canvas.clear();

        let display_green = match timer.elapsed() {
            Ok(elapsed) => elapsed.as_secs() % 2 == 0,
            Err(_) => true
        };

        let square_texture = if display_green {&green_square} else {&blue_square};
        canvas.copy(
            square_texture,
            None,
            Rect::new(0, 0, TEXTURE_SIZE, TEXTURE_SIZE)
        ).expect("Could not copy texture to canvas");
        
        canvas.copy(&image_texture, None, None).expect("Could not copy image texture to canvas");

        canvas.present();

        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

}
