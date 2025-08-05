use macroquad::prelude::{Color, Vec2};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct MyVec2 {
    pub x: f32,
    pub y: f32,
}

impl MyVec2
{
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x, 
            y: y
        }
    }
}

pub fn from_vec2(v: Vec2) -> MyVec2 {
    MyVec2 { x: v.x, y: v.y }
}

pub fn to_vec2(v: MyVec2) -> Vec2 {
    Vec2 { x: v.x, y: v.y }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct MyColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn from_color(c: Color) -> MyColor {
    MyColor {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

pub fn to_color(c: MyColor) -> Color {
    Color::new(c.r, c.g, c.b, c.a)
}
