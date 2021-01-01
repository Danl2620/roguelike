use super::{Map, Monster, Position, Viewshed, RunState, WantsToMelee};
use specs::prelude::*;
//use rltk::{field_of_view,Point,console};
use rltk::{Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        // ReadStorage<'a, Name>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewsheds,
            monsters,
            mut positions,
            mut wants_to_melee) = data;

        for (entity, mut viewshed, _monster, mut position) in
            (&entities, &mut viewsheds, &monsters, &mut positions).join()
        {
            if *runstate != RunState::MonsterTurn { return; }

            if rltk::DistanceAlg::Pythagoras
                    .distance2d(Point::new(position.x, position.y), *player_pos)
                    < 1.5
            {
                //console::log(&format!("{} shouts insults", name.name));
                wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("unable to insert attack");
            }
            else if viewshed.visible_tiles.contains(&*player_pos) {
                let path = rltk::a_star_search(
                    map.xy_idx(position.x, position.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    let pos = map.size.idx_position(path.steps[1]);
                    position.x = pos.x;
                    position.y = pos.y;
                    viewshed.dirty = true;
                    //console::log(&format!("{} runs towards you", name.name));
                }
            }
        }
    }
}
