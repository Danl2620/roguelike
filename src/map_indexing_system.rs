use specs::prelude::*;
use super::{Map,Position,BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a,Map>,
                        ReadStorage<'a,Position>,
                        ReadStorage<'a,BlocksTile>,
                        Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &positions).join() {
            let idx = map.position_idx(*position);

            // if the entity blocks update the block map
            let blocker : Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = blocker {
                map.blocked[idx] = true;
            }

            // add the entity to the tile (it's a copy type so no clone needed)
            map.tile_content[idx].push(entity);
        }
    }
}