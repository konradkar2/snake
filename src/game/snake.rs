use crate::config::{SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::prelude::{Color, Vec2, draw_rectangle, vec2};

pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) struct Snake {
    positions: Vec<Vec2>,
    previous_tail_position: Vec2,
    direction: Direction,
    size: f32,
    speed: f32,
    color: Color,
}

impl Snake {
    pub(crate) fn new(size: f32, speed: f32, color: Color) -> Self {
        let last_tail_pos = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);

        let mut ret = Self {
            direction: Direction::Left,
            previous_tail_position: last_tail_pos,
            positions: Vec::from([last_tail_pos]),
            size: size,
            speed: speed,
            color: color,
        };

        ret.move_step();
        ret.grow();

        ret
    }

    pub(crate) fn move_step(&mut self) {
        let tail_pos = self.positions.last_mut().unwrap();
        self.previous_tail_position = *tail_pos;

        if self.positions.len() > 1 {
            for i in (1..self.positions.len()).rev() {
                self.positions[i] = self.positions[i - 1];
            }
        }

        let head_pos = &mut self.positions[0];

        match self.direction {
            Direction::Up => head_pos.y -= self.speed,
            Direction::Down => head_pos.y += self.speed,
            Direction::Left => head_pos.x -= self.speed,
            Direction::Right => head_pos.x += self.speed,
        }

        if head_pos.x > SCREEN_WIDTH {
            head_pos.x = 0.0;
        } else if head_pos.x < 0.0 {
            head_pos.x = SCREEN_WIDTH;
        }

        if head_pos.y > SCREEN_HEIGHT {
            head_pos.y = 0.0;
        } else if head_pos.y < 0.0 {
            head_pos.y = SCREEN_HEIGHT;
        }
    }

    pub(crate) fn grow(&mut self) {
        self.positions.push(self.previous_tail_position);
    }

    pub(crate) fn draw(&self) {
        for pos in &self.positions {
            draw_rectangle(pos.x, pos.y, self.size, self.size, self.color);
        }
    }

    pub(crate) fn change_direction(&mut self, direction: Direction) {
        match self.direction {
            Direction::Left | Direction::Right => match direction {
                Direction::Down | Direction::Up => self.direction = direction,
                _ => {}
            },
            Direction::Up | Direction::Down => match direction {
                Direction::Left | Direction::Right => self.direction = direction,
                _ => {}
            },
        }
    }

    pub(crate) fn collides_self(&self) -> bool {
        let head_pos: Vec2 = self.positions[0];

        for i in 1..self.positions.len() {
            if self.positions[i] == head_pos {
                return true;
            }
        }
        false
    }

    pub(crate) fn collides_fruit(&self, fruit: &Vec2) -> bool {
        for i in 0..self.positions.len() {
            if self.positions[i] == *fruit {
                return true;
            }
        }
        false
    }
}
