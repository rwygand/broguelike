mod indexing_system;

use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;
pub use indexing_system::MapIndexingSystem;

const MAPWIDTH : usize = 80;
const MAPHEIGHT : usize = 50;
const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(Component, Debug, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn from_point(p: Point) -> Position {
        Position { x: p.x, y: p.y }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx:usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }

    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    fn new(depth: i32, width: i32, height: i32) -> Self {
        Self {
            depth,
            width,
            height,
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked : vec![false; MAPCOUNT],
            tile_content : vec![Vec::new(); MAPCOUNT],
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn new_map_rooms_and_corridors(_depth: i32, width: i32, height: i32) -> Map {
        let mut map = Map::new(1, width, height);

        let mut rng = RandomNumberGenerator::new();
        map.apply_rooms_and_corridors(&mut rng);
        map
    }

    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    #[allow(dead_code)]
    pub fn set_tile(&mut self, idx: usize, tile: TileType) {
        self.tiles[idx] = tile;
    }

    pub fn set_tile_xy(&mut self, x: i32, y: i32, tile: TileType) {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] = tile;
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    #[allow(dead_code)]
    pub fn idx_xy(&self, idx: usize) -> (usize, usize) {
        let x = idx % self.width as usize;
        let y = match idx {
            0 => 0,
            _ => idx / self.width as usize,
        };
        (x, y)
    }

    pub fn center_of_room(&self, room_idx: i32) -> Point {
        self.rooms[room_idx as usize].center()
    }

    fn apply_rooms_and_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                self.apply_room(&new_room);

                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center().to_tuple();
                    let (prev_x, prev_y) = self.center_of_room(self.rooms.len() as i32-1).to_tuple();
                    if rng.range(0, 2) == 1 {
                        self.apply_horizontal_tunnel( prev_x, new_x, prev_y);
                        self.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.apply_vertical_tunnel( prev_y, new_y, prev_x);
                        self.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);
            }
        }
    }

    fn apply_room(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                self.set_tile_xy(x, y, TileType::Floor);
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let min = x1.min(x2);
        let max = x1.max(x2);

        for x in min..=max {
            self.set_tile_xy(x, y, TileType::Floor);
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        let min = y1.min(y2);
        let max = y1.max(y2);

        for y in min..=max {
            self.set_tile_xy(x, y, TileType::Floor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xy_idx() {
        let gen = Map::new(1, 80, 50);
        assert_eq!(gen.xy_idx(0, 0), 0);
        assert_eq!(gen.xy_idx(1, 0), 1);
        assert_eq!(gen.xy_idx(0, 1), 80);
        assert_eq!(gen.xy_idx(2, 10), 802);
        assert_eq!(gen.xy_idx(79, 49), 3999);
    }

    #[test]
    fn test_idx_xy() {
        let gen = Map::new(1, 80, 50);

        assert_eq!(gen.idx_xy(0), (0, 0));
        assert_eq!(gen.idx_xy(1), (1, 0));
        assert_eq!(gen.idx_xy(80), (0, 1));
        assert_eq!(gen.idx_xy(802), (2, 10));
        assert_eq!(gen.idx_xy(3999), (79, 49));
    }
}
