use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::*;
use crate::gamelog::GameLog;
use crate::map::Position;

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount : i32
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToDrinkPotion {
    pub potion : Entity
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToDropItem {
    pub item : Entity
}

pub fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}
