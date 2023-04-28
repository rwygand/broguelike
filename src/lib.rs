pub mod states;
pub mod player;
pub mod map;
pub mod visibility;
pub mod monster;
pub mod combat;
pub mod render;
pub mod gui;
pub mod gamelog;
pub mod spawner;
pub mod items;
pub mod inventory_system;

use specs::prelude::*;
use specs_derive::*;
use crate::combat::{DamageSystem, MeleeCombatSystem};
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, PotionUseSystem};
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
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut potions = PotionUseSystem{};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain();
    }
}