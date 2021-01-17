use super::{
    BlocksTile, CombatStats, Consumable, Item, Monster, Name, Player, Position, ProvidesHealing,
    Rect, Renderable, Viewshed,
};
use rand::seq::SliceRandom;
use rand_core::{impls, Error, RngCore};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const MAX_MONSTERS_PER_ROOM: i32 = 2;
const MAX_ITEMS_PER_ROOM: i32 = 4;

// ------------------------------------------------------------------------------------------------------------------ //
pub struct SpawnContext<'a> {
    pub world: &'a mut World,
    pub rng: &'a mut RandomNumberGenerator,
    pub position: Position,
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn player(context: &mut SpawnContext) -> Entity {
    context
        .world
        .create_entity()
        .with(context.position)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build()
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn random_monster(context: &mut SpawnContext) -> Entity {
    match context.rng.roll_dice(1, 2) {
        1 => orc(&mut context.world, &context.position),
        _ => goblin(&mut context.world, &context.position),
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn orc(world: &mut World, position: &Position) -> Entity {
    monster(world, position, rltk::to_cp437('o'), "Orc")
}

// ------------------------------------------------------------------------------------------------------------------ //
fn goblin(world: &mut World, position: &Position) -> Entity {
    monster(world, position, rltk::to_cp437('g'), "Goblin")
}

// ------------------------------------------------------------------------------------------------------------------ //
fn monster<S: ToString>(
    world: &mut World,
    position: &Position,
    glyph: rltk::FontCharType,
    name: S,
) -> Entity {
    world
        .create_entity()
        .with(*position)
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build()
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn health_potion(world: &mut World, position: &Position) -> Entity {
    world
        .create_entity()
        .with(*position)
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { amount: 8 })
        .build()
}

//#[derive(Debug)]
struct RngWrapper<'a> {
    rng: &'a mut RandomNumberGenerator,
}

impl RngCore for RngWrapper<'_> {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn spawn_room(world: &mut World, rng: &mut RandomNumberGenerator, room: &Rect) {
    let mut room_positions = (0..room.area()).collect::<Vec<_>>();
    room_positions.shuffle(&mut RngWrapper { rng });

    let spawn_items = |count: i32| -> Vec<Position> {
        let mut points: Vec<Position> = Vec::new();
        let point_count = i32::min(count, room.area());
        for ii in 0..point_count {
            points.push(room.idx_position(room_positions[ii as usize] as usize));
        }
        points
    };

    let monster_points = spawn_items(rng.range(0, MAX_MONSTERS_PER_ROOM));
    let item_points = spawn_items(rng.range(0, MAX_ITEMS_PER_ROOM));

    for p in monster_points.iter() {
        random_monster(&mut SpawnContext {
            world: world,
            rng: rng,
            position: *p,
        });
    }

    for p in item_points.iter() {
        health_potion(world, p);
    }
}
