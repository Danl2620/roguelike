use rltk::{GameState, Point, Rltk};
use specs::prelude::*;
mod components;
use components::*;
mod map;
use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod viewport;
pub use viewport::Viewport;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod behavior;
use behavior::MonsterAI;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod gamelog;
pub use gamelog::GameLog;
mod gui;
//use gui::ItemMenuResult;
mod inventory_system;
mod spawner;
use inventory_system::{ItemCollectionSystem, ItemUseSystem};

// ------------------------------------------------------------------------------------------------------------------ //
pub struct State {
    pub viewport: Viewport,
    pub ecs: World,
}

// ------------------------------------------------------------------------------------------------------------------ //
impl State {
    fn run_systems(&mut self) {
        let mut use_items = ItemUseSystem {};
        use_items.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut combat = MeleeCombatSystem {};
        combat.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        damage_system::delete_the_dead(&mut self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust Roguelike!");

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        let (rs, should_show_inventory) = match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                (RunState::AwaitingInput, false)
            }
            RunState::AwaitingInput => (player_input(self, ctx), false),
            RunState::PlayerTurn => {
                self.run_systems();
                (RunState::MonsterTurn, false)
            }
            RunState::MonsterTurn => {
                self.run_systems();
                (RunState::AwaitingInput, false)
            }
            RunState::ShowInventory => {
                let result = gui::menu_inventory(self, ctx);

                match result {
                    gui::ItemMenuResult::Cancel => (RunState::AwaitingInput, false),
                    gui::ItemMenuResult::NoResponse => (newrunstate, true),
                    gui::ItemMenuResult::Selected(entity) => {
                        let mut intent = self.ecs.write_storage::<WantsToConsumeItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToConsumeItem { item: entity },
                            )
                            .expect("Unable to insert intent");
                        (RunState::PlayerTurn, false)
                    }
                }
            }
        };

        newrunstate = rs;

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate
        }

        damage_system::delete_the_dead(&mut self.ecs);

        let map = self.ecs.fetch::<Map>();
        map.draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx, &self.viewport, should_show_inventory);
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    // create world
    let mut world = World::new();
    world.register::<Position>();
    world.register::<CombatStats>();
    world.register::<Renderable>();
    world.register::<Viewshed>();
    world.register::<Name>();
    world.register::<BlocksTile>();
    world.register::<Player>();
    world.register::<Monster>();
    world.register::<WantsToMelee>();
    world.register::<SufferDamage>();
    world.register::<Item>();
    world.register::<Consumable>();
    world.register::<ProvidesHealing>();
    world.register::<InBackpack>();
    world.register::<WantsToPickupItem>();
    world.register::<WantsToConsumeItem>();

    //world.insert(new_map(&gs));
    let viewport = Viewport {
        map_width: 80,
        map_height: 43,
        log_height: 7,
    };
    let mut rng = rltk::RandomNumberGenerator::seeded(1);
    let map = Map::new_map_rooms_and_corridors(&mut world, &viewport, &mut rng);
    let (px, py) = map.rooms[0].center();

    // for room in map.rooms.iter().skip(1) {
    //     let (x,y) = room.center();

    //     let mut context = spawner::SpawnContext {
    //         world: &mut world,
    //         rng: &mut rng,
    //         position: Position{x,y},
    //     };

    //     // create a room monster per room
    //     spawner::random_monster(&mut context);
    // }

    world.insert(map);

    // create the player!
    let player_entity = spawner::player(&mut spawner::SpawnContext {
        world: &mut world,
        rng: &mut rng,
        position: Position { x: px, y: py },
    });

    world.insert(Point::new(px, py));
    world.insert(player_entity);
    world.insert(RunState::PreRun);
    world.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    // create game state
    let gs = State {
        viewport: viewport,
        ecs: world,
    };

    // start app
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);
    rltk::main_loop(context, gs)
}
