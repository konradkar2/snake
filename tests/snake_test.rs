#![allow(dead_code)]

#[path = "../src/common.rs"]
mod common;

#[path = "../src/snake_cfg.rs"]
mod snake_cfg;

#[path = "../src/game/snake.rs"]
mod snake;

use common::{MyColor, MyVec2};
use snake::{Direction, Snake, SnakeCollision};
use snake_cfg::{SNAKE_SIZE, SNAKE_TICKS_PER_MOVE};

const START_POS: MyVec2 = MyVec2 { x: 100.0, y: 100.0 };
const START_DIRECTION: Direction = Direction::Left;

fn test_color() -> MyColor {
    MyColor {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

fn pos(x: f32, y: f32) -> MyVec2 {
    MyVec2::new(x, y)
}

fn new_test_snake() -> Snake {
    let snake = Snake::new(test_color(), START_POS);
    assert_eq!(snake.get_direction(), START_DIRECTION);
    snake
}

fn move_ticks(snake: &mut Snake) {
    for _ in 0..SNAKE_TICKS_PER_MOVE as u32 {
        snake.move_step_tick();
    }
}

#[test]
fn new_snake_starts_with_head_and_tail_positions() {
    let snake = new_test_snake();

    assert!(snake.collides_object(&pos(START_POS.x - SNAKE_SIZE, START_POS.y)));
    assert!(snake.collides_object(&START_POS));
    assert!(!snake.collides_object(&pos(START_POS.x + SNAKE_SIZE, START_POS.y)));
}

#[test]
fn move_step_tick_moves_only_after_configured_number_of_ticks() {
    let mut snake = new_test_snake();

    for _ in 0..(SNAKE_TICKS_PER_MOVE as u32 - 1) {
        snake.move_step_tick();
    }

    assert!(snake.collides_object(&pos(START_POS.x - SNAKE_SIZE, START_POS.y)));
    assert!(!snake.collides_object(&pos(START_POS.x - SNAKE_SIZE * 2.0, START_POS.y)));

    snake.move_step_tick();

    assert!(snake.collides_object(&pos(START_POS.x - SNAKE_SIZE * 2.0, START_POS.y)));
}

#[test]
fn change_direction_rejects_reverse_direction() {
    let mut snake = new_test_snake();

    snake.change_direction(Direction::Right);
    move_ticks(&mut snake);

    assert_eq!(snake.get_direction(), Direction::Left);
}

#[test]
fn change_direction_accepts_perpendicular_direction() {
    let mut snake = new_test_snake();

    snake.change_direction(Direction::Up);
    move_ticks(&mut snake);

    assert_eq!(snake.get_direction(), Direction::Up);
    assert!(snake.collides_object(&pos(START_POS.x - SNAKE_SIZE, START_POS.y - SNAKE_SIZE)));
}

#[test]
fn grow_keeps_previous_tail_after_next_move() {
    let mut snake = new_test_snake();

    snake.grow();
    move_ticks(&mut snake);

    assert!(snake.collides_object(&START_POS));
}

#[test]
fn detects_head_to_head_collision_with_other_snake() {
    let snake = new_test_snake();
    let other = new_test_snake();

    match snake.collides_other(&other) {
        Some(SnakeCollision::HeadToHead) => {}
        _ => panic!("expected head-to-head collision"),
    }
}

#[test]
fn detects_head_to_tail_collision_with_other_snake() {
    let snake = Snake::new(test_color(), pos(START_POS.x + SNAKE_SIZE, START_POS.y));
    let other = new_test_snake();

    match snake.collides_other(&other) {
        Some(SnakeCollision::HeadToTail) => {}
        _ => panic!("expected head-to-tail collision"),
    }
}

#[test]
fn detects_self_collision() {
    let mut snake = new_test_snake();

    for _ in 0..4 {
        snake.grow();
    }

    move_ticks(&mut snake);
    snake.change_direction(Direction::Down);
    move_ticks(&mut snake);
    snake.change_direction(Direction::Right);
    move_ticks(&mut snake);
    snake.change_direction(Direction::Up);
    move_ticks(&mut snake);

    assert!(snake.collides_self());
}
