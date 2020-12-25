use specs::prelude::*;
use super::{Viewshed,Monster,Name};
//use rltk::{field_of_view,Point,console};
use rltk::{console,Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = ( ReadExpect<'a, Point>, 
                        ReadStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>
                        );
    
    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewsheds, monsters, name) = data;

        for (viewshed,_monster,name) in (&viewsheds, &monsters, &name).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} shouts insults", name.name));
            }
        }
    }
}