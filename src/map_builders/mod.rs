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
mod voronoi;
mod waveform_collpase;

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
        voronoi::VoronoiCellBuilder,
        waveform_collpase::WaveformCollapseBuilder
    }
};

const MAP_TESTING_MODE: bool = true;

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
            Box::new(SimpleMapBuilder::new(new_depth))
        }
        false => {
            let mut rng = RandomNumberGenerator::new();
            let builder = rng.roll_dice(1, 16);
            let mut result : Box<dyn MapBuilder>;
            match builder {
                1 => result = Box::new(BspDungeonBuilder::new(new_depth)),
                2 => result = Box::new(BspInteriorBuilder::new(new_depth)),
                3 => result = Box::new(CellularAutomataBuilder::new(new_depth)),
                4 => result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
                5 => result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
                6 => result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
                7 => result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
                8 => result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
                9 => result = Box::new(MazeBuilder::new(new_depth)),
                10 => result = Box::new(DLABuilder::walk_inwards(new_depth)),
                11 => result = Box::new(DLABuilder::walk_outwards(new_depth)),
                12 => result = Box::new(DLABuilder::central_attractor(new_depth)),
                13 => result = Box::new(DLABuilder::insectoid(new_depth)),
                14 => result = Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
                15 => result = Box::new(VoronoiCellBuilder::manhattan(new_depth)),
                _ => result = Box::new(SimpleMapBuilder::new(new_depth))
            }
            if rng.roll_dice(1, 3)==1 {
                result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
            }
            result
        }
    }
}