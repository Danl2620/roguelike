use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

// ------------------------------------------------------------------------------------------------------------------ //
pub fn range<T: std::cmp::Ord>(l: T, v: T, u: T) -> T {
    min(u, max(v, l))
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

// // ------------------------------------------------------------------------------------------------------------------ //
// pub fn new_map_rooms_and_corridors() -> Vec<TileType> {
// 	let mut map = vec!{TilesType::Wall, }
// }

// ------------------------------------------------------------------------------------------------------------------ //
/// Make a map with solid boundaries and 400 randomly placed walls
fn new_map(gs: &State) -> Vec<TileType> {
    let sx = gs.size.0;
    let sy = gs.size.1;
    let mut map = vec![TileType::Floor; (sx * sy) as usize];

    for x in 0..sx {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, sy - 1)] = TileType::Wall;
    }

    for y in 0..sy {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(sx - 1, y)] = TileType::Wall;
    }

    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, sx - 1);
        let y = rng.roll_dice(1, sy - 1);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
struct Player {}

struct State {
    size: (i32, i32),
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

struct LeftWalker {}

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

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
fn try_move_player(delta_x: i32, delta_y: i32, gs: &State) {
    let mut ecs = &gs.ecs;
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let (nx, ny) = (
            range(0, pos.x + delta_x, gs.size.0 - 1),
            range(0, pos.y + delta_y, gs.size.1 - 1),
        );
        let destination_idx = xy_idx(nx, ny);
        if map[destination_idx] == TileType::Floor {
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
fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        let (color, character) = match tile {
            TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), '.'),
            TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), '#'),
        };

        ctx.set(
            x,
            y,
            color,
            RGB::from_f32(0., 0., 0.),
            rltk::to_cp437(character),
        );

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
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
    gs.ecs.register::<Player>();

    gs.ecs.insert(new_map(&gs));

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }

    rltk::main_loop(context, gs);
}
