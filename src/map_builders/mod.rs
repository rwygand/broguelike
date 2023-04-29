use bracket_lib::prelude::RandomNumberGenerator;
use specs::World;

mod simple_map;
mod common;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod drunkard;
mod maze;
mod dla;

use crate::{
    Position, Map,
    map_builders::{
        common::*,
        simple_map::SimpleMapBuilder,
        bsp_dungeon::BspDungeonBuilder,
        bsp_interior::BspInteriorBuilder,
        cellular_automata::CellularAutomataBuilder,
        drunkard::DrunkardsWalkBuilder,
        maze::MazeBuilder,
        dla::DLABuilder,
    }
};

const MAP_TESTING_MODE: bool = false;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs : &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    match MAP_TESTING_MODE {
        true => {
            // if we're testing, render the last testing map
            Box::new(MazeBuilder::new(new_depth))
        }
        false => {
            let mut rng = RandomNumberGenerator::new();
            let builder = rng.roll_dice(1, 12);
            match builder {
                1 => Box::new(BspDungeonBuilder::new(new_depth)),
                2 => Box::new(BspInteriorBuilder::new(new_depth)),
                3 => Box::new(CellularAutomataBuilder::new(new_depth)),
                4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
                5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
                6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
                7 => Box::new(MazeBuilder::new(new_depth)),
                8 => Box::new(DLABuilder::walk_inwards(new_depth)),
                9 => Box::new(DLABuilder::walk_outwards(new_depth)),
                10 => Box::new(DLABuilder::central_attractor(new_depth)),
                11 => Box::new(DLABuilder::insectoid(new_depth)),
                _ => Box::new(SimpleMapBuilder::new(new_depth))
            }
        }
    }
}