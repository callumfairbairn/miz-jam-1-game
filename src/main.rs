mod level;
mod update;
mod event;
mod tile;
mod grid;
mod constants;

mod entity;

use nannou::{
    prelude::*,
    image::open
};


use constants::{WINDOW_RES_X, WINDOW_RES_Y};
use grid::Grid;
use event::event;
use update::update;
use level::{generate_level, hearts};
use entity::{
    PlayerInstance,
    Instance,
    EnvironmentState
};

pub struct Model {
    grid: Grid,
    tile_tex: nannou::wgpu::Texture,

    player: PlayerInstance,
    env: EnvironmentState,
}

impl Model {
    pub fn tick(&mut self) {
        self.player.tick(&self.env);
    }
}

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    app.new_window().size(WINDOW_RES_X as u32, WINDOW_RES_Y as u32).event(event).view(view).build().unwrap();

    let tile_image = open(app.assets_path().unwrap().join("tilesheet.png")).unwrap();
    let tile_tex = wgpu::Texture::from_image(app, &tile_image);

    let level = generate_level(hearts());
    let grid = Grid::new_from_level(level, &tile_tex.size());
    let player = PlayerInstance::new();

    let env = EnvironmentState::new();

    Model {
        grid,
        tile_tex,

        player,
        env
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    // Draw background...
    // TODO: set sample descriptor so its not all blurry
    draw.mesh().tris_textured(&model.tile_tex, model.grid.vertices.clone());

    // Draw player...
    //draw.mesh().tris_textured(&model.player.tile ?, model.grid.vertices.clone());

    // Finish
    draw.to_frame(app, &frame).unwrap();
}