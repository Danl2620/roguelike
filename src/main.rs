use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

mod map;
pub use crate::map::{Map, TileType};

mod visibility_system;
use visibility_system::VisibilitySystem;

// ------------------------------------------------------------------------------------------------------------------ //
pub fn range<T: std::cmp::Ord>(l: T, v: T, u: T) -> T {
    min(u, max(v, l))
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
pub struct Position {
    x: i32,
    y: i32,
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
struct LeftMover {}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Player {}

// ------------------------------------------------------------------------------------------------------------------ //
struct State {
    size: (i32, i32),
    ecs: World,
}

// ------------------------------------------------------------------------------------------------------------------ //
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Map>();
        map.draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
struct LeftWalker {}

// ------------------------------------------------------------------------------------------------------------------ //
impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn try_move_player(delta_x: i32, delta_y: i32, gs: &State) {
    let ecs = &gs.ecs;
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let (nx, ny) = (
            range(0, pos.x + delta_x, gs.size.0 - 1),
            range(0, pos.y + delta_y, gs.size.1 - 1),
        );
        let destination_idx = map.xy_idx(nx, ny);
        if map.tiles[destination_idx] == TileType::Floor {
            pos.x = nx;
            pos.y = ny;
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // player movement
    match ctx.key {
        None => {}
        Some(key) => {
            let (dx, dy) = match key {
                VirtualKeyCode::Left => (-1, 0),
                VirtualKeyCode::Right => (1, 0),
                VirtualKeyCode::Up => (0, -1),
                VirtualKeyCode::Down => (0, 1),
                _ => (0, 0),
            };
            try_move_player(dx, dy, gs);
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn main() {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();
    let mut gs = State {
        size: (80, 50),
        ecs: World::new(),
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Player>();

    //gs.ecs.insert(new_map(&gs));
    let map = Map::new_map_rooms_and_corridors(gs.size.0, gs.size.1);
    let (px, py) = map.rooms[0].center();

    gs.ecs.insert(map);

    // create the player!
    gs.ecs
        .create_entity()
        .with(Position { x: px, y: py })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .build();

    // for i in 0..10 {
    //     gs.ecs
    //         .create_entity()
    //         .with(Position { x: i * 7, y: 20 })
    //         .with(Renderable {
    //             glyph: rltk::to_cp437('☺'),
    //             fg: RGB::named(rltk::RED),
    //             bg: RGB::named(rltk::BLACK),
    //         })
    //         .with(LeftMover {})
    //         .build();
    // }

    rltk::main_loop(context, gs);
}
