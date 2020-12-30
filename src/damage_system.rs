use specs::prelude::*;
use rltk::{console};
use super::{CombatStats,SufferDamage,Name,Player,GameLog};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut statses, mut damages) = data;

        for (mut stats, damage) in (&mut statses, &damages).join() {
            stats.hp = i32::max(0, stats.hp - damage.amount.iter().sum::<i32>());
        }

        damages.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp == 0 { 
                let target_name = names.get(entity).unwrap();
                let player = players.get(entity);
                match player {
                    None => {
                        log.entries.push(format!("{} has died!", &target_name.name));
                        dead.push(entity);
                    }
                    Some(_) => log.entries.push("You have died!".to_string())
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead");
    }
}