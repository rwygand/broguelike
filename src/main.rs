use bracket_lib::prelude::*;
use specs::prelude::*;
use broguelike_rltk::map::{Map, Position};
use broguelike_rltk::monster::Monster;
use broguelike_rltk::player::Player;
use broguelike_rltk::{BlocksTile, Name};
use broguelike_rltk::combat::{CombatStats, SufferDamage, WantsToMelee};
use broguelike_rltk::render::Renderable;
use broguelike_rltk::states::RunState;
use broguelike_rltk::visibility::Viewshed;

const WIDTH: usize = 80;
const HEIGHT: usize = 50;

embedded_resource!(TILE_FONT, "../resources/monochrome-transparent_packed.png");

pub const SPRITE_SIZE: usize = 16;
pub const SPRITE_SHEET_COLS: usize = 49;
pub const SPRITE_SHEET_ROWS: usize = 22;

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

    let mut gs = broguelike_rltk::State {
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
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(Point::new(player_point.x, player_point.y));
    gs.ecs.insert(player_entity);

    main_loop(context, gs)
}

pub fn sprite_at(row: usize, col: usize) -> u16 {
    (SPRITE_SHEET_COLS as u16 * row as u16) + col as u16
}
