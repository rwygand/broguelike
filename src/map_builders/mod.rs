mod simple_map;
mod common;
mod bsp_dungeon;
mod bsp_interior;

use specs::World;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::Position;
use super::Map;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs : &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}


pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // let mut rng = RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 2);
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth))
    // }
    Box::new(BspInteriorBuilder::new(new_depth))
}