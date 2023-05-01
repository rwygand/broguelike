use bracket_lib::prelude::*;
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, WantsToUseItem,
    Consumable, ProvidesHealing, WantsToDropItem, InflictsDamage, Map, SufferDamage,
    AreaOfEffect, Confusion, Equippable, Equipped, WantsToRemoveItem, particle_system::ParticleBuilder,
    ProvidesFood, HungerClock, HungerState, MagicMapper, RunState, Pools, EquipmentChanged,
    TownPortal, IdentifiedItem, Item, ObfuscatedName, MagicItem, MasterDungeonMap};

fn obfuscate_name(
    item: Entity, 
    names: &ReadStorage::<Name>, 
    magic_items : &ReadStorage::<MagicItem>,
    obfuscated_names : &ReadStorage::<ObfuscatedName>,
    dm : &MasterDungeonMap,
) -> String 
{
    if let Some(name) = names.get(item) {
        if magic_items.get(item).is_some() {
            if dm.identified_items.contains(&name.name) {
                name.name.clone()
            } else if let Some(obfuscated) = obfuscated_names.get(item) {
                obfuscated.name.clone()
            } else {
                "Unidentified magic item".to_string()
            }
        } else {
            name.name.clone()
        }

    } else {
        "Nameless item (bug)".to_string()
    }
}

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                        ReadStorage<'a, MagicItem>,
                        ReadStorage<'a, ObfuscatedName>,
                        ReadExpect<'a, MasterDungeonMap>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names,
            mut backpack, mut dirty, magic_items, obfuscated_names, dm) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(
                    format!(
                        "You pick up the {}.", 
                        obfuscate_name(pickup.item, &names, &magic_items, &obfuscated_names, &dm)
                    )
                );
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, ProvidesHealing>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteStorage<'a, Pools>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
                        WriteExpect<'a, ParticleBuilder>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, ProvidesFood>,
                        WriteStorage<'a, HungerClock>,
                        ReadStorage<'a, MagicMapper>,
                        WriteExpect<'a, RunState>,
                        WriteStorage<'a, EquipmentChanged>,
                        ReadStorage<'a, TownPortal>,
                        WriteStorage<'a, IdentifiedItem>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut wants_use, names,
            consumables, healing, inflict_damage, mut combat_stats, mut suffer_damage,
            aoe, mut confused, equippable, mut equipped, mut backpack, mut particle_builder, positions,
            provides_food, mut hunger_clocks, magic_mapper, mut runstate, mut dirty, town_portal,
            mut identified_item) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");
            let mut used_item = true;

            // Targeting
            let mut targets : Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push( *player_entity ); }
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            // Single target in tile
                            let idx = map.xy_idx(target.x, target.y);
                            crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob) );
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles = field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                                particle_builder.request(tile_idx.x, tile_idx.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('░'), 200.0);
                            }
                        }
                    }
                }
            }

            // Identify
            if entity == *player_entity {
                identified_item.insert(entity, IdentifiedItem{ name: names.get(useitem.item).unwrap().name.clone() })
                    .expect("Unable to insert");
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Remove any items the target has in the item's slot
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                    }

                    // Wield the item
                    equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            // It it is edible, eat it!
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    used_item = true;
                    let target = targets[0];
                    let hc = hunger_clocks.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!("You eat the {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            // If its a magic mapper...
            let is_mapper = magic_mapper.get(useitem.item);
            match is_mapper {
                None => {}
                Some(_) => {
                    used_item = true;
                    gamelog.entries.push("The map is revealed to you!".to_string());
                    *runstate = RunState::MagicMapReveal{ row : 0};
                }
            }

            // If its a town portal...
            if let Some(_townportal) = town_portal.get(useitem.item) {
                if map.depth == 1 {
                    gamelog.entries.push("You are already in town, so the scroll does nothing.".to_string());
                    used_item = false;
                } else {
                    used_item = true;
                    gamelog.entries.push("You are telported back to town!".to_string());
                    *runstate = RunState::TownPortal;
                }
            }

            // If it heals, apply the healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    used_item = false;
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hit_points.current = i32::min(stats.hit_points.max, stats.hit_points.current + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You use the {}, healing {} hp.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                            used_item = true;

                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, RGB::named(GREEN), RGB::named(BLACK), to_cp437('♥'), 200.0);
                            }
                        }
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage, true);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, inflicting {} hp.", item_name.name, mob_name.name, damage.damage));

                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, RGB::named(RED), RGB::named(BLACK), to_cp437('‼'), 200.0);
                            }
                        }

                        used_item = true;
                    }
                }
            }

            // Can it pass along confusion? Note the use of scopes to escape from the borrow checker!
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns ));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!("You use {} on {}, confusing them.", item_name.name, mob_name.name));

                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(pos.x, pos.y, RGB::named(MAGENTA), RGB::named(BLACK), to_cp437('?'), 200.0);
                                }
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion{ turns: mob.1 }).expect("Unable to insert status");
            }

            // If its a consumable, we delete it on use
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                        ReadStorage<'a, MagicItem>,
                        ReadStorage<'a, ObfuscatedName>,
                        ReadExpect<'a, MasterDungeonMap>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions,
            mut backpack, mut dirty, magic_items, obfuscated_names, dm) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            if entity == *player_entity {
                gamelog.entries.push(
                    format!(
                        "You drop the {}.",
                        obfuscate_name(to_drop.item, &names, &magic_items, &obfuscated_names, &dm)
                    )
                );
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
                        Entities<'a>,
                        WriteStorage<'a, WantsToRemoveItem>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}

pub struct ItemIdentificationSystem {}

impl<'a> System<'a> for ItemIdentificationSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
                        ReadStorage<'a, crate::components::Player>,
                        WriteStorage<'a, IdentifiedItem>,
                        WriteExpect<'a, crate::map::MasterDungeonMap>,
                        ReadStorage<'a, Item>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, ObfuscatedName>,
                        Entities<'a>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player, mut identified, mut dm, items, names, mut obfuscated_names, entities) = data;

        for (_p, id) in (&player, &identified).join() {
            if !dm.identified_items.contains(&id.name) && crate::raws::is_tag_magic(&id.name) {
                dm.identified_items.insert(id.name.clone());

                for (entity, _item, name) in (&entities, &items, &names).join() {
                    if name.name == id.name {
                        obfuscated_names.remove(entity);
                    }
                }
            }
        }

        // Clean up
        identified.clear();
    }
}
