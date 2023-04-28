use bracket_lib::geometry::Point;
use specs::prelude::*;
use crate::map::{Map, Position};
use super::Viewshed;
use bracket_lib::pathfinding::field_of_view;
use crate::player::Player;


pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height );

            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height );

                // If this is the player, reveal what they can see
                if player.get(ent).is_some() {
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}