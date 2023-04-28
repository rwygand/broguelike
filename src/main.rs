mod player;
mod map;
mod visibility;
mod monster;
mod combat;

use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;
use crate::map::{Map, Position, TileType};
use crate::player::*;
use crate::visibility::Viewshed;
use visibility::system::VisibilitySystem;
use crate::monster::Monster;
use combat::*;

const WIDTH: usize = 80;
const HEIGHT: usize = 50;

embedded_resource!(TILE_FONT, "../resources/monochrome-transparent_packed.png");

pub const SPRITE_SIZE: usize = 16;
pub const SPRITE_SHEET_COLS: usize = 49;
pub const SPRITE_SHEET_ROWS: usize = 22;
pub fn sprite_at(row: usize, col: usize) -> u16 {
    (SPRITE_SHEET_COLS as u16 * row as u16) + col as u16
}

fn main() -> BError {
    link_resource!(TILE_FONT, "resources/monochrome-transparent_packed.png");

    let context = BTermBuilder::new()
        .with_dimensions(WIDTH as u32, HEIGHT as u32)
        .with_tile_dimensions(16u32, 16u32)
        .with_title("Broguelike")
        .with_font("monochrome-transparent_packed.png", 16u32, 16u32)
        //.with_simple_console(WIDTH as u32, HEIGHT as u32, "monochrome-transparent_packed.png")
        .with_sparse_console_no_bg(WIDTH as u32, HEIGHT as u32, "monochrome-transparent_packed.png")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        runstate : RunState::Running
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map = Map::new_map_rooms_and_corridors(1, 80, 50);
    let player_point = map.center_of_room(0);

    let player_entity = gs.ecs.create_entity()
        .with(Position { x: player_point.x, y: player_point.y } )
        .with(Renderable {
            glyph: 25,
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK)
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Player{})
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build();

    let mut rng = RandomNumberGenerator::new();

    for (i,room) in map.rooms.iter().skip(1).enumerate() {
        let p = room.center();

        let glyph : FontCharType;
        let name : String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = sprite_at(2, 29); name = "Goblin".to_string() }
            _ => { glyph = sprite_at(2, 26); name = "Orc".to_string() }
        }

        gs.ecs.create_entity()
            .with(Position::from_point(p))
            .with(Renderable{
                glyph: glyph,
                fg: RGB::named(RED),
                bg: RGB::named(BLACK),
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .with(Name{ name: format!("{} #{}", &name, i) })
            .with(BlocksTile{})
            .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_point.x, player_point.y));
    gs.ecs.insert(player_entity);

    main_loop(context, gs)
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

pub struct State {
    ecs: World,
    runstate: RunState,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = monster::MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = map::MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        combat::delete_the_dead(&mut self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut BTerm) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
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

