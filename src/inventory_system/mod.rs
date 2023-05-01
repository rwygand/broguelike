pub mod use_system;
pub mod remove_system;
pub mod identification_system;
pub mod drop_system;
pub mod collection_system;
pub mod equip_use;

use specs::prelude::*;

use crate::components::*;
use crate::map::MasterDungeonMap;

pub use use_system::ItemUseSystem;
pub use remove_system::ItemRemoveSystem;
pub use identification_system::ItemIdentificationSystem;
pub use drop_system::ItemDropSystem;
pub use collection_system::ItemCollectionSystem;
pub use equip_use::ItemEquipOnUse;

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