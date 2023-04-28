use specs::prelude::*;
use crate::visibility::system::VisibilitySystem;
use crate::monster::MonsterAI;
use crate::map::MapIndexingSystem;
use crate::combat::{
    MeleeCombatSystem,
    DamageSystem,
    delete_the_dead
};

impl crate::State {
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

        delete_the_dead(&mut self.ecs);

        self.ecs.maintain();
    }
}