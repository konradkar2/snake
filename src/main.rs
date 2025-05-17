use std::ops::Index;

use macroquad::prelude::*;

const SNAKE_SPEED: f32 = 1.0;

struct Snake {
    positions: Vec<Vec2>,
    direction: Direction,
}

impl Snake {
    fn new() -> Self {
        Self {
            direction: Direction::Left,
            positions: Vec::from([vec2(screen_width() / 2.0, screen_height() / 2.0)]),
        }
    }

    fn move_step(&mut self) {
        for i in 1..self.positions.len() {
            let previous_pos = self.positions.index(i - 1);
            self.positions[i] = *previous_pos;
        }

        let head = &mut self.positions[0];

        match self.direction {
            Direction::Up => head.y -= SNAKE_SPEED,
            Direction::Down => head.y += SNAKE_SPEED,
            Direction::Left => head.x -= SNAKE_SPEED,
            Direction::Right => head.x += SNAKE_SPEED,
        }
    }

    fn draw(&self) {
        for pos in &self.positions {
            draw_rectangle(pos.x, pos.y, 10.0, 10.0, GREEN);
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut snake = Snake::new();

    loop {
        clear_background(RED);

        //draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        let char = get_char_pressed();

        if char.is_some() {
            match char.unwrap() {
                'w' => snake.direction = Direction::Up,
                's' => snake.direction = Direction::Down,
                'a' => snake.direction = Direction::Left,
                'd' => snake.direction = Direction::Right,
                _ => {}
            }
        }

        snake.move_step();
        snake.draw();

        next_frame().await
    }
}
