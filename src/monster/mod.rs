mod ai_system;

use specs::prelude::*;
use specs_derive::*;
use crate::visibility::Viewshed;
pub use ai_system::MonsterAI;

#[derive(Component, Debug)]
pub struct Monster {}