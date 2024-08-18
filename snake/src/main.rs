pub mod snake;
pub mod food;

use raylib::prelude::*;
use snake::{Direction, Snake};
use food::Food;
use std::{collections::VecDeque, time::{Duration, Instant}};

const SCREEN_WIDTH : i32 = 800;
const SCREEN_HEIGHT: i32 = 800;
const BLOCK_PER_ROW: i32 = 32; // need to be multiple of screen_widith and height
const BLOCK_PER_COL: i32 = 32;
const SNAKE_SPEED: Duration = Duration::from_millis(100);

fn main() {

    // states
    let initial_position = vec![(4,3), (3,3), (2,3)];
    let mut snake = Snake::new(VecDeque::from(initial_position.clone()));
    let mut food = Food {pos: (10,10), have_food: true};

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Hello world")
        .build();

    let mut last_update = Instant::now();
    let mut start_game = false;

    while !rl.window_should_close() {

        //game events
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            start_game = true;
        }

        if !food.have_food {
            food.pos = get_new_pos(&mut rl, &snake);
            food.have_food = true;
        }

        // snake events
        change_direction(&mut rl, &mut snake);
        if last_update.elapsed() >= SNAKE_SPEED && start_game {
            snake.move_snake();
            last_update = Instant::now();
        }
        let (hx, hy) = snake.new_head();
        if hx < 0 || hx >= BLOCK_PER_ROW || hy < 0 || hy == BLOCK_PER_COL {
            start_game = false;
            snake = Snake::new(VecDeque::from(initial_position.clone()));
        }
        if food.have_food && (hx, hy) == food.pos {
            food.have_food = false;
            snake.is_eaten = true;
        }

        if snake.is_block_in_body(snake.new_head()) {
            start_game = false;
            snake = Snake::new(VecDeque::from(initial_position.clone()));
        }

        // draw events
        draw_game(&mut rl.begin_drawing(&thread), &snake, &food);
    }
}

fn get_new_pos(rl: &mut RaylibHandle, snake: &Snake) -> (i32, i32) {
    let mut new_pos : (i32, i32) = 
        (rl.get_random_value(1..BLOCK_PER_ROW - 2), rl.get_random_value( 1..BLOCK_PER_COL - 2));
    while snake.is_block_in_body(new_pos) {
        new_pos = (rl.get_random_value(1..BLOCK_PER_ROW - 2), rl.get_random_value( 1..BLOCK_PER_COL - 2));
    }
    new_pos
}

fn draw_game(d: &mut RaylibDrawHandle, snake: &Snake, food: &Food) {
    d.clear_background(Color::WHITE);
    draw_boarder(d);
    draw_snake(d, &snake);
    draw_food(d, &food);
}

fn draw_food(d: &mut RaylibDrawHandle, food: &Food) {
    if food.have_food {
        draw_block(d, food.pos.0, food.pos.1, Color::RED);
    }
}

fn change_direction(rl: &mut RaylibHandle, snake: &mut Snake) {
    match rl.get_key_pressed() {
        Some(KeyboardKey::KEY_UP) if snake.direction != Direction::DOWN => snake.direction = Direction::UP,
        Some(KeyboardKey::KEY_DOWN) if snake.direction != Direction::UP => snake.direction = Direction::DOWN,
        Some(KeyboardKey::KEY_RIGHT) if snake.direction != Direction::LEFT => snake.direction = Direction::RIGHT,
        Some(KeyboardKey::KEY_LEFT) if snake.direction != Direction::RIGHT => snake.direction = Direction::LEFT,
        _ => {}
}

}

fn draw_snake(d: &mut RaylibDrawHandle, snake: &Snake) {
    for (x, y) in &snake.body {
        draw_block(d, *x, *y, Color::GREENYELLOW);
    }
}

fn draw_boarder(d: &mut RaylibDrawHandle) {
    let color = Color::DARKGRAY;
    for n in 0..2 {
        for i in 0..BLOCK_PER_ROW {
            draw_block(d,i,n * (BLOCK_PER_COL - 1),color);
        }
        for i in 0..BLOCK_PER_COL {
            draw_block(d,(BLOCK_PER_ROW - 1) * n,i,color);
        }
    }
}

fn draw_block(d: &mut RaylibDrawHandle, x: i32, y: i32, color: Color) {
    let width = SCREEN_WIDTH / BLOCK_PER_ROW;
    let height = SCREEN_HEIGHT / BLOCK_PER_COL;
    d.draw_rectangle(x * width, y * height, width, height, color);
}