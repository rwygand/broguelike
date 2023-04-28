use specs::prelude::*;
use bracket_lib::prelude::*;
use crate::combat::CombatStats;
use crate::map::{MAPWIDTH, Position};
use crate::monster::Monster;
use crate::{BlocksTile, Name};
use crate::items::{Item, Potion};
use crate::player::Player;
use crate::render::Renderable;
use crate::visibility::Viewshed;

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;

/// Fills a room with stuff!
pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();
    let mut item_spawn_points : Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH as usize) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0 .. num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH as usize) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH as usize;
        let y = *idx / MAPWIDTH as usize;
        random_monster(ecs, Point::new(x, y));
    }
    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH as usize;
        let y = *idx / MAPWIDTH as usize;
        health_potion(ecs, x as i32, y as i32);
    }

}

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_point: Point) -> Entity {
    ecs.create_entity()
        .with(Position { x: player_point.x, y: player_point.y } )
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Player{})
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, point: Point) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, point) }
        _ => { goblin(ecs, point) }
    }
}

fn orc(ecs: &mut World, point: Point) { monster(ecs, point, to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, point: Point) { monster(ecs, point, to_cp437('g'), "Goblin"); }

fn monster<S : ToString>(ecs: &mut World, point: Point, glyph : FontCharType, name : S) {
    ecs.create_entity()
        .with(Position::from_point(point))
        .with(Renderable{
            glyph,
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 1,
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(Potion{ heal_amount: 8 })
        .build();
}