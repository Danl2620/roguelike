use rltk::{GameState, Rltk, RGB, Point};
use specs::prelude::*;
mod components;
pub use components::*;
mod map;
pub use map::*;
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
mod gui;
mod gamelog;
pub use gamelog::GameLog;

// ------------------------------------------------------------------------------------------------------------------ //
pub struct State {
    pub viewport: Viewport,
    pub ecs: World,
}

// ------------------------------------------------------------------------------------------------------------------ //
impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut combat = MeleeCombatSystem{};
        combat.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        damage_system::delete_the_dead(&mut self.ecs);
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

        newrunstate = 
            match newrunstate {
                RunState::PreRun => {
                    self.run_systems();
                    RunState::AwaitingInput
                }
                RunState::AwaitingInput => {
                    player_input(self, ctx)
                }
                RunState::PlayerTurn => {
                    self.run_systems();
                    RunState::MonsterTurn
                }
                RunState::MonsterTurn => {
                    self.run_systems();
                    RunState::AwaitingInput
                }
            };

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate
        }

        damage_system::delete_the_dead(&mut self.ecs);

        let map = self.ecs.fetch::<Map>();
        map.draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx, &self.viewport);
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

    //world.insert(new_map(&gs));
    let viewport = Viewport { map_width: 80, map_height: 43, log_height: 7 };
    let map = Map::new_map_rooms_and_corridors(&viewport);
    let (px, py) = map.rooms[0].center();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (i,room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();

        let roll = rng.roll_dice(1,2);
        let (c,name) = match roll {
            1 => { ('g', "Goblin".to_string()) }
            _ => { ('o', "Orc".to_string()) }
        };
        let glyph: rltk::FontCharType = rltk::to_cp437(c);

        // create a room monster per room
        world.create_entity()
            .with(Position{x,y})
            .with(Renderable{
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true})
            .with(Monster{})
            .with(CombatStats{max_hp:16, hp: 16, defense: 1, power: 4})
            .with(Name{ name: format!("{} #{}", &name, i) })
            .with(BlocksTile{})
            .build();
    }

    world.insert(map);

    // create the player!
    let player_entity = world
        .create_entity()
        .with(Position { x: px, y: py })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5})
        .with(Name{ name: "Player".to_string() })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        })
        .build();

    world.insert(Point::new(px, py));
    world.insert(player_entity);
    world.insert(RunState::PreRun);
    world.insert(gamelog::GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });

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
