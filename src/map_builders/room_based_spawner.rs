use super::{MetaMapBuilder, BuilderMap, spawner};
use bracket_lib::prelude::RandomNumberGenerator;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(&build_data.map, rng, room, build_data.map.depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("Room Based Spawning only works after rooms have been created");
        }
    }
}