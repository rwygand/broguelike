pub mod states;
pub mod player;
pub mod map;
pub mod visibility;
pub mod monster;
pub mod combat;
pub mod render;

use specs::prelude::*;
use specs_derive::*;
use crate::combat::{DamageSystem, MeleeCombatSystem};
use crate::map::MapIndexingSystem;
use crate::monster::MonsterAI;
use crate::visibility::system::VisibilitySystem;

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct BlocksTile {}


pub struct State {
    pub ecs: World,
}

impl State {
    pub(crate) fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}