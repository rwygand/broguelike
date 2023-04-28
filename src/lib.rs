pub mod systems;
pub mod states;
pub mod player;
pub mod map;
pub mod visibility;
pub mod monster;
pub mod combat;
pub mod render;

use specs::prelude::*;
use specs_derive::*;
use crate::states::RunState;

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct BlocksTile {}


pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}