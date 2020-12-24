use rltk::{ RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point };
use super::{Rect, Viewshed};
use std::cmp::{max, min};
use specs::prelude::*;

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>
}

impl Map {
    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn new_map_rooms_and_corridors(size_x: i32, size_y: i32) -> Map {
        let sx = size_x;
        let sy = size_y;

        let mut map = Map {
            tiles: vec![TileType::Wall; (sx * sy) as usize],
            rooms: Vec::new(),
            width: sx,
            height: sy,
            revealed_tiles: vec![false; sx as usize * sy as usize],
            visible_tiles: vec![false; sx as usize * sy as usize]
        };

        const MAX_ROOMS: i32 = 30;
        const SIZE_RANGE: (i32, i32) = (6, 10);

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let (w, h) = (
                rng.range(SIZE_RANGE.0, SIZE_RANGE.1),
                rng.range(SIZE_RANGE.0, SIZE_RANGE.1),
            );
            let (x, y) = (
                rng.roll_dice(1, sx - w - 1) - 1,
                rng.roll_dice(1, sy - h - 1) - 1,
            );
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    let (tunnel_x, tunnel_y) = if rng.range(0, 2) == 1 {
                        (new_x, prev_y)
                    } else {
                        (prev_x, new_y)
                    };
                    map.apply_vertical_tunnel(prev_y, new_y, tunnel_x);
                    map.apply_horizontal_tunnel(prev_x, new_x, tunnel_y);
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn draw_map(&self, ecs: &World, ctx: &mut Rltk) {
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        //let mut players = ecs.write_storage::<Player>();
        //let map = ecs.fetch::<Map>();

        for _viewshed in (&mut viewsheds).join() {
            let mut y = 0;
            let mut x = 0;
            for (idx, tile) in self.tiles.iter().enumerate() {
                //render a tile depending on the tile type

                if self.revealed_tiles[idx] {
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
                }

                // let pt = Point::new(x, y);
                // if viewshed.visible_tiles.contains(&pt) {
                //     let (color, character) = match tile {
                //         TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), '.'),
                //         TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), '#'),
                //     };

                //     ctx.set(
                //         x,
                //         y,
                //         color,
                //         RGB::from_f32(0., 0., 0.),
                //         rltk::to_cp437(character),
                //     );
                // }

                x += 1;
                if x > 79 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    // Make a map with solid boundaries and 400 randomly placed walls
    // fn new_map(&self, gs: &State) -> Vec<TileType> {
    //     let sx = gs.size.0;
    //     let sy = gs.size.1;
    //     let mut map = vec![TileType::Floor; (sx * sy) as usize];

    //     for x in 0..sx {
    //         map[self.xy_idx(x, 0)] = TileType::Wall;
    //         map[self.xy_idx(x, sy - 1)] = TileType::Wall;
    //     }

    //     for y in 0..sy {
    //         map[self.xy_idx(0, y)] = TileType::Wall;
    //         map[self.xy_idx(sx - 1, y)] = TileType::Wall;
    //     }

    //     let mut rng = rltk::RandomNumberGenerator::new();

    //     for _i in 0..400 {
    //         let x = rng.roll_dice(1, sx - 1);
    //         let y = rng.roll_dice(1, sy - 1);
    //         let idx = self.xy_idx(x, y);
    //         if idx != self.xy_idx(40, 25) {
    //             map[idx] = TileType::Wall;
    //         }
    //     }

    //     map
    // }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}
