use bracket_lib::prelude::*;
use super::{MapBuilder, Map, Position};
use RandomNumberGenerator;
use specs::prelude::*;
use crate::{SHOW_MAPGEN_VISUALIZER, spawner, TileType};
use common::*;

use crate::map_builders::common;

pub struct SimpleMapBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>
}

impl MapBuilder for SimpleMapBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl SimpleMapBuilder {
    pub fn new(new_depth : i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
        }
    }

    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(&mut self.map, &new_room);
                self.take_snapshot();

                if !self.rooms.is_empty() {
                    let new_pos = new_room.center();
                    let prev_pos = self.rooms[self.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_pos.x, new_pos.x, prev_pos.y);
                        apply_vertical_tunnel(&mut self.map, prev_pos.y, new_pos.y, new_pos.x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_pos.y, new_pos.y, prev_pos.x);
                        apply_horizontal_tunnel(&mut self.map, prev_pos.x, new_pos.x, new_pos.y);
                    }
                }

                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }

        let stairs_position = self.rooms[self.rooms.len()-1].center();
        let stairs_idx = self.map.xy_idx(stairs_position.x, stairs_position.y);
        self.map.tiles[stairs_idx] = TileType::StairsDown;

        let start_pos = self.rooms[0].center();
        self.starting_position = Position{ x: start_pos.x, y: start_pos.y };
    }
}