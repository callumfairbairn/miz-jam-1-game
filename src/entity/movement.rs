use bitflags::bitflags;
use crate::level::Level;
use crate::constants::{COLLISION_MULTIPLIER, TILE_RES};
use crate::level::Side::{LEFT, RIGHT, TOP, BOTTOM};
use rand::Rng;

bitflags! {
    #[derive(Default)]
    pub struct Direction: u8 {
        const UP = 0b0001;
        const DOWN = 0b0010;
        const LEFT = 0b0100;
        const RIGHT = 0b1000;
    }
}
#[derive(Clone, Copy)]
enum YMove {
    None,
    Up,
    Down
}
#[derive(Clone, Copy)]
enum XMove {
    None,
    Left,
    Right
}

impl Direction {
    fn reduce(self) -> (XMove, YMove) {
        let y = match (self & (Direction::UP | Direction::DOWN)).bits() {
            0b00 | 0b11 => YMove::None,
            0b01 => YMove::Up,
            0b10 => YMove::Down,
            _ => unreachable!()
        };

        let x = match (self & (Direction::LEFT | Direction::RIGHT)).bits() >> 2 {
            0b00 | 0b11 => XMove::None,
            0b01 => XMove::Left,
            0b10 => XMove::Right,
            _ => unreachable!()
        };

        (x, y)
    }
}

pub fn random_direction() -> Direction {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(0, 4);
    if num == 0 { return Direction::UP };
    if num == 1 { return Direction::DOWN };
    if num == 2 { return Direction::LEFT };
    Direction::RIGHT
}

pub fn distance_between(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0)).sqrt()
}

// Defines the core attributes of an entity's movement.
pub struct MovementAttributes {
    pub attack: f64,  // 0->(sustain) in (attack) ticks
    pub sustain: f64, // (sustain) pixels per tick
    pub release: f64, // (sustain)->0 in (release) ticks
}

// Defines the state of a entity in space.
#[derive(Clone)]
pub struct MovementState {
    x: f64,
    y: f64,

    x_velo: f64,
    y_velo: f64,
}

impl MovementState {
    pub fn new((x, y): (f64, f64)) -> Self {
        Self {
            x: x,
            y: y,

            x_velo: 0.0,
            y_velo: 0.0
        }
    }

    pub fn tick(&mut self, attrs: &MovementAttributes, apply_direction: Direction, level: &Level) {
        let (x_move, y_move) = apply_direction.reduce();

        let (max_speed_x, max_speed_y) = match (x_move, y_move) {
            (XMove::None, YMove::None) => (0.0, 0.0),
            (_, YMove::None) => (attrs.sustain, 0.0),
            (XMove::None, _) => (0.0, attrs.sustain),
            (_, _) => {
                let max_speed = (attrs.sustain.powi(2) / 2.0).sqrt();
                (max_speed, max_speed)
            }
        };
        let decel = attrs.sustain / attrs.release;
        let accel = attrs.sustain / attrs.attack;

        let x_velo = match x_move {
            XMove::None => if self.x_velo > 0.0 {   // Release while travelling right
                let new_x_velo = self.x_velo - decel;
                if new_x_velo < 0.0 {0.0} else {new_x_velo}
            } else if self.x_velo < 0.0 {               // Release while travelling left
                let new_x_velo = self.x_velo + decel;
                if new_x_velo > 0.0 {0.0} else {new_x_velo}
            } else {
                0.0
            },
            XMove::Left => if self.x_velo > 0.0 {   // Release when travelling right
                self.x_velo - decel
            } else if self.x_velo > -max_speed_x {  // Accelerate left
                let new_x_velo = self.x_velo - accel;
                if new_x_velo < -max_speed_x {-max_speed_x} else {new_x_velo}
            } else if self.x_velo < -max_speed_x {  // Release when above max speed
                self.x_velo + accel
            } else {    // Maintain max speed
                -max_speed_x
            },
            XMove::Right => if self.x_velo < 0.0 {
                self.x_velo + decel
            } else if self.x_velo < max_speed_x {  // Accelerate right
                let new_x_velo = self.x_velo + accel;
                if new_x_velo > max_speed_x {max_speed_x} else {new_x_velo}
            } else if self.x_velo > max_speed_x {  // Release when above max speed
                self.x_velo - accel
            } else {
                max_speed_x
            },
        };

        let y_velo = match y_move {
            YMove::None => if self.y_velo > 0.0 {   // Release while travelling down
                let new_y_velo = self.y_velo - decel;
                if new_y_velo < 0.0 {0.0} else {new_y_velo}
            } else if self.y_velo < 0.0 {               // Release while travelling up
                let new_y_velo = self.y_velo + accel;
                if new_y_velo > 0.0 {0.0} else {new_y_velo}
            } else {
                0.0
            },
            YMove::Down => if self.y_velo < 0.0 {
                self.y_velo + decel
            } else if self.y_velo < max_speed_y {
                let new_y_velo = self.y_velo + accel;
                if new_y_velo > max_speed_y {max_speed_y} else {new_y_velo}
            } else if self.y_velo > max_speed_y {
                self.y_velo - accel
            } else {
                max_speed_y
            },
            YMove::Up => if self.y_velo > 0.0 {
                self.y_velo - decel
            } else if self.y_velo > -max_speed_y {
                let new_y_velo = self.y_velo - accel;
                if new_y_velo < -max_speed_y {-max_speed_y} else {new_y_velo}
            } else if self.y_velo < -max_speed_y {
                self.y_velo + accel
            } else {
                -max_speed_y
            },
        };

        let new_x = self.x + x_velo;
        let new_y = self.y + y_velo;
        let new_x_velo = x_velo;
        let new_y_velo = y_velo;

        let player_rect = crate::rect::Rect{
            pos: (new_x as f32, new_y as f32),
            size: (1.0, 1.0)
        };

        let mut collision = false;
        let mut collision_directions = Vec::new();

        for (y, row) in level.floor.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.is_some() {
                    let tile = tile.as_ref().unwrap();
                    if tile.solid {
                        let tile_rect = crate::rect::Rect {
                            pos: (x as f32, y as f32),
                            size: (1.0, 1.0),
                        };

                        if player_rect.collides_with(&tile_rect) {
                            collision = true;
                            collision_directions.push(player_rect.get_nearest_wall(&tile_rect));
                        }
                    }
                }
            }
        }

        let push_out_of_wall_amount = 0.25 / TILE_RES as f64;

        if collision {
            if collision_directions.contains(&TOP) || collision_directions.contains(&BOTTOM) {
                if new_y_velo == 0.0 {
                    if collision_directions.contains(&TOP) {
                        self.y = new_y + push_out_of_wall_amount;
                    } else {
                        self.y = new_y - push_out_of_wall_amount;
                    }
                } else {
                    self.y_velo = -new_y_velo * COLLISION_MULTIPLIER;
                }
            } else {
                self.y = new_y;
                self.y_velo = new_y_velo;
            }
            if collision_directions.contains(&LEFT) || collision_directions.contains(&RIGHT) {
                if new_x_velo == 0.0 {
                    if collision_directions.contains(&LEFT) {
                        self.x = self.x - push_out_of_wall_amount;
                    } else {
                        self.x = self.x + push_out_of_wall_amount;
                    }
                } else {
                    self.x_velo = -new_x_velo * COLLISION_MULTIPLIER;
                }
            } else {
                self.x = new_x;
                self.x_velo = new_x_velo;
            }
        } else {
            self.x = new_x;
            self.y = new_y;
            self.x_velo = new_x_velo;
            self.y_velo = new_y_velo;
        }
    }

    pub fn x_pos(&self) -> f32 {
        self.x as f32
    }

    pub fn y_pos(&self) -> f32 {
        self.y as f32
    }
}
