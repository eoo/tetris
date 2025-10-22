extern crate sdl3;
extern crate rand;

mod tetrimino;
mod tetris;

use crate::tetris::Tetris;

use std::alloc::System;
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
const NUM_HIGHSCORES: usize = 5;

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

fn update_highscore_vec(v: &mut Vec<u32>, value: u32) -> bool {
    if v.len() < NUM_HIGHSCORES {
        v.push(value);
        v.sort();
        true
    } else {
        for entry in v.iter_mut() {
            if value > *entry {
                *entry = value;
                v.sort();
                return true;    
            }
        }
        false
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

fn handle_events(
    tetris: &mut Tetris,
    quit: &mut bool,
    timer: &mut SystemTime,
    event_pump: &mut sdl3:: EventPump
) -> bool {
    let mut make_permanent = false;

    let Some(ref mut tetrimino) = tetris.current_tetrimino else {
        return make_permanent;
    };

    let mut tmp_x = tetrimino.x;
    let mut tmp_y = tetrimino.y;

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                *quit = true;
                break;
            },
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                *timer = SystemTime::now();
                tmp_y += 1;
            },
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                tmp_x -= 1;
            },
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                tmp_x += 1;
            },
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                tetrimino.rotate(&tetris.game_map);
            },
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                let x = tetrimino.x;
                let mut y = tetrimino.y;

                while tetrimino.change_position(&tetris.game_map, x, y + 1) == true {
                    y += 1;
                }
                make_permanent = true;
            },
            _ => {}
        }
    }
    
    if !make_permanent {
        if tetrimino.change_position(&tetris.game_map, tmp_x, tmp_y) == false && tmp_y != tetrimino.y {
            make_permanent = true;
        }
    }

    if make_permanent {
        tetris.make_permanent();
        *timer = SystemTime::now();
    }
    make_permanent
}

fn print_game_information(tetris: &Tetris) {
    println!("Game Over...");
    println!("Score: {}", tetris.score);
    // println!("Lines: {}", tetris.number_of_lines);
    println!("Current Level: {}", tetris.current_level);
    // chere highscore and update if needed
}

fn is_time_over(tetris: &Tetris, timer: &SystemTime) -> bool {
    match timer.elapsed() {
        Ok(elapsed) => {
            let millis = elapsed.as_millis() as u32 + elapsed.subsec_millis();
            millis > tetris::LEVEL_TIMES[tetris.current_level as usize - 1]
        },
        Err(_) => false
    }
}

pub fn main() {
    let sdl_context = sdl3::init().expect("SDL initialization failed");
    // let video_subsystem = sdl_context.video().expect("Could not get SDL video subsystem");

    // let window = video_subsystem.window("Tetris", 800, 600)
    //     .position_centered()
    //     .opengl()
    //     .build()
    //     .expect("Could not create window");
    
    // let mut canvas = window.into_canvas();
    
    // let texture_creator = canvas.texture_creator();
    
    let mut event_pump = sdl_context.event_pump().expect("Could not get SDL event pump");

    let mut tetris = Tetris::new();
    let mut timer = SystemTime::now();

    let grid_x = (width - TETRIS_HEIGHT as u32 * 10) as i32 / 2;
    let grid_y = (height - TETRIS_HEIGHT as u32 * 16) as i32 / 2;
    
    loop {
        if is_time_over(&tetris, &timer) {
            let mut make_permanent = false;
            if let Some(ref mut tetrimino) = tetris.current_tetrimino {
                let x = tetrimino.x;
                let y = tetrimino.y + 1;
                make_permanent = !tetrimino.change_position(&tetris.game_map, x, y);
            }
            if make_permanent {
                tetris.make_permanent();
            }
            timer = SystemTime::now();
        }

        // draw tetris grid here

        if tetris.current_tetrimino.is_none() {
            let current_tetrimino = tetris.create_new_tetrimino();
            if !current_tetrimino.test_current_position(&tetris.game_map) {
                print_game_information(&tetris);
                break
            }
            tetris.current_tetrimino = Some(current_tetrimino);    
        }

        let mut quit = false;
        if !handle_events(&mut tetris, &mut quit, &mut timer, &mut event_pump) {
            if let Some(ref mut tetrimino) = tetris.current_tetrimino {
               // Drawing code here
            }
        }
        if quit {
            print_game_information(&tetris);
            break
        }

        // Present the updated canvas

        sleep(Duration::new(0, 1_000_000u32 / 60));
    }

}
