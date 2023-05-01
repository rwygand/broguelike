pub mod item_use_system;
pub mod item_remove_system;
pub mod item_identification_system;
pub mod item_drop_system;
pub mod item_collection_system;

use specs::prelude::*;

use crate::components::*;
use crate::map::MasterDungeonMap;

pub use item_use_system::ItemUseSystem;
pub use item_remove_system::ItemRemoveSystem;
pub use item_identification_system::ItemIdentificationSystem;
pub use item_drop_system::ItemDropSystem;
pub use item_collection_system::ItemCollectionSystem;

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