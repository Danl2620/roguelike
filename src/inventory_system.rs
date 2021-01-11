use super::{
    gamelog::GameLog, CombatStats, Consumable, InBackpack, Name, Position, ProvidesHealing,
    WantsToConsumeItem, WantsToPickupItem,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToConsumeItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_consume,
            names,
            consumables,
            healings,
            mut combat_stats,
        ) = data;

        for (entity, consume, stats) in (&entities, &wants_consume, &mut combat_stats).join() {
            let heal_item = healings.get(consume.item);
            match heal_item {
                None => {}
                Some(heal) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + heal.amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(consume.item).unwrap().name,
                            heal.amount
                        ));
                    }

                    let consumable = consumables.get(consume.item);
                    match consumable {
                        None => {}
                        Some(_) => {
                            entities.delete(consume.item).expect("Delete failed");
                        }
                    }
                }
            }
        }
        wants_consume.clear();
    }
}
