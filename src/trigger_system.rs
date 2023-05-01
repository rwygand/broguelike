use specs::prelude::*;
use super::{EntityMoved, Position, EntryTrigger, Hidden, Map, Name, gamelog::GameLog,
    InflictsDamage, particle_system::ParticleBuilder, SufferDamage, SingleActivation,
    TeleportTo, ApplyTeleport};
use bracket_lib::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        WriteStorage<'a, Hidden>,
                        ReadStorage<'a, Name>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, SingleActivation>,
                        ReadStorage<'a, TeleportTo>,
                        WriteStorage<'a, ApplyTeleport>,
                        ReadExpect<'a, Entity>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, mut hidden,
            names, entities, mut log, inflicts_damage, mut particle_builder,
            mut inflict_damage, single_activation, teleporters,
            mut apply_teleport, player_entity) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities : Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id { // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(entity_id);
                            if let Some(name) = name {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }

                            hidden.remove(entity_id); // The trap is no longer hidden

                            // If the trap is damage inflicting, do it
                            let damage = inflicts_damage.get(entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(pos.x, pos.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('‼'), 200.0);
                                SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage, false);
                            }

                            // If its a teleporter, then do that
                            if let Some(teleport) = teleporters.get(entity_id) {
                                if (teleport.player_only && entity == *player_entity) || !teleport.player_only {
                                    apply_teleport.insert(entity, ApplyTeleport{
                                        dest_x : teleport.x,
                                        dest_y : teleport.y,
                                        dest_depth : teleport.depth
                                    }).expect("Unable to insert");
                                }
                            }

                            // If it is single activation, it needs to be removed
                            let sa = single_activation.get(entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(entity_id);
                            }
                        }
                    }
                }
            });
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
