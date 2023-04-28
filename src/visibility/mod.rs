pub mod system;

use specs::prelude::*;
use specs_derive::*;
use bracket_lib::prelude::*;

#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles : Vec<Point>,
    pub range : i32,
    pub dirty : bool
}