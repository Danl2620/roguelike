use super::{Map, Monster, Name, Position, Viewshed};
use specs::prelude::*;
//use rltk::{field_of_view,Point,console};
use rltk::{console, Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewsheds, monsters, names, mut positions) = data;

        for (mut viewshed, _monster, name, mut position) in
            (&mut viewsheds, &monsters, &names, &mut positions).join()
        {
            if viewshed.visible_tiles.contains(&*player_pos) {
                let path = rltk::a_star_search(
                    map.xy_idx(position.x, position.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map,
                );
                if path.success {
                    if path.steps.len() > 1 {
                        position.x = path.steps[1] as i32 % map.width;
                        position.y = path.steps[1] as i32 / map.width;
                        viewshed.dirty = true;
                        console::log(&format!("{} runs towards you", name.name));
                    }
                } else if rltk::DistanceAlg::Pythagoras
                    .distance2d(Point::new(position.x, position.y), *player_pos)
                    < 1.5
                {
                    console::log(&format!("{} shouts insults", name.name));
                }
            }
        }
    }
}
