mod player;
mod map;
mod visibility;

use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;
use crate::map::{Map, Position, TileType};
use crate::player::*;
use crate::visibility::Viewshed;
use visibility::system::VisibilitySystem;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 50;

embedded_resource!(TILE_FONT, "../resources/monochrome-transparent_packed.png");

fn main() -> BError {
    link_resource!(TILE_FONT, "resources/monochrome-transparent_packed.png");

    let context = BTermBuilder::new()
        .with_dimensions(WIDTH as u32, HEIGHT as u32)
        .with_tile_dimensions(16u32, 16u32)
        .with_title("Broguelike")
        .with_font("monochrome-transparent_packed.png", 16u32, 16u32)
        .with_sparse_console(WIDTH as u32, HEIGHT as u32, "monochrome-transparent_packed.png")
        .build()?;

    let mut gs = State {
        ecs: World::new()
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors(1, 80, 50);
    let player_point = map.center_of_room(0);

    gs.ecs.create_entity()
        .with(Position { x: player_point.x, y: player_point.y } )
        .with(Renderable {
            glyph: 25,
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK)
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Player{})
        .build();

    gs.ecs.insert(map);

    main_loop(context, gs)
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}


pub struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut BTerm) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

pub fn draw_map(ecs: &World, ctx : &mut BTerm) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx,tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type

        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = 0;
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = 16;
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }
            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
