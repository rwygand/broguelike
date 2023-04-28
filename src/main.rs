mod player;
mod map;

use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;
use crate::map::{Map, Position, TileType};
use crate::player::*;

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



    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: 7*49+18,
                fg: RGB::named(RED),
                bg: RGB::named(BLACK),
            })
            .build();
    }

    let map = Map::new_map_rooms_and_corridors(1, 80, 50);
    let player_point = map.center_of_room(0);

    gs.ecs.create_entity()
        .with(Position { x: player_point.x, y: player_point.y } )
        .with(Renderable {
            glyph: 25,
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK)
        })
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

impl GameState for State {
    fn tick(&mut self, ctx : &mut BTerm) {
        player_input(self, ctx);
        ctx.cls();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        draw_map(&map, ctx);

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}


pub fn player_input(gs: &mut State, ctx: &mut BTerm) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::X => try_move_player(0, 1, &mut gs.ecs),

            _ => {}
        },
    }
}

fn draw_map(map: &Map, ctx : &mut BTerm) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.tiles.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), 0);
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.2, 0.1, 0.05), RGB::from_f32(0., 0., 0.), 16);
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}