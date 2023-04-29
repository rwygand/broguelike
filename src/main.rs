use bracket_lib::prelude::*;
use specs::prelude::*;
use broguelike::map::{Map, MAPHEIGHT, MAPWIDTH, Position};
use broguelike::monster::Monster;
use broguelike::player::Player;
use broguelike::{BlocksTile, Name, spawner};
use broguelike::combat::{CombatStats, SufferDamage, WantsToMelee};
use broguelike::render::Renderable;
use broguelike::states::RunState;
use broguelike::visibility::Viewshed;
use broguelike::gamelog::GameLog;
use broguelike::items::{Consumable, InBackpack, Item, ProvidesHealing, WantsToUseItem, WantsToDropItem, WantsToPickupItem, InflictsDamage, Ranged, AreaOfEffect, Confusion};

//embedded_resource!(TILE_FONT, "../resources/monochrome-transparent_packed.png");
//embedded_resource!(TEXT_FONT, "../resources/terminal_10x16.png");

pub const SPRITE_SIZE: usize = 16;
pub const SPRITE_SHEET_COLS: usize = 49;
pub const SPRITE_SHEET_ROWS: usize = 22;

fn main() -> BError {
    //link_resource!(TILE_FONT, "resources/monochrome-transparent_packed.png");
    //link_resource!(TEXT_FONT, "resources/terminal_10x16.png");

    let mut context = BTermBuilder::simple80x50()
        //.with_dimensions(WIDTH as u32, HEIGHT as u32)
        .with_tile_dimensions(16u32, 16u32)
        .with_title("Broguelike")
        .with_fps_cap(30.)
        //.with_font("monochrome-transparent_packed.png", 16u32, 16u32)
        //.with_font("terminal_10x16.png", 16u32, 16u32)
        //.with_simple_console(WIDTH, HEIGHT, "monochrome-transparent_packed.png")
        //.with_sparse_console_no_bg(WIDTH, HEIGHT,"terminal_10x16.png")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = broguelike::State {
        ecs: World::new(),
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();

    gs.ecs.insert(RandomNumberGenerator::new());

    let map = Map::new_map_rooms_and_corridors(1, MAPWIDTH, MAPHEIGHT);
    let player_point = map.center_of_room(0);
    let player_entity = spawner::player(&mut gs.ecs, player_point);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(player_point);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });

    main_loop(context, gs)
}

pub fn sprite_at(row: usize, col: usize) -> u16 {
    (SPRITE_SHEET_COLS as u16 * row as u16) + col as u16
}
