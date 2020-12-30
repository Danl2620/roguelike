use rltk::{ RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point };
use super::{Rect, Viewshed, Position, Viewport};
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
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>
}

impl Map {
    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn position_idx(&self, p:Position) -> usize {
        self.xy_idx(p.x,p.y)
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x,y);
        !self.blocked[idx]
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
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
    pub fn new_map_rooms_and_corridors(viewport: &Viewport) -> Map {
        let sx = viewport.map_width;
        let sy = viewport.map_height;
        let vec_size = (sx * sy) as usize;

        let mut map = Map {
            tiles: vec![TileType::Wall; (sx * sy) as usize],
            rooms: Vec::new(),
            width: sx,
            height: sy,
            revealed_tiles: vec![false; vec_size],
            visible_tiles: vec![false; vec_size],
            blocked: vec![false; vec_size],
            tile_content: vec![Vec::new(); vec_size]
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
                rng.roll_dice(1, sx - w - 2),
                rng.roll_dice(1, sy - h - 2),
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
                    let (visible_color, character) = match tile {
                        TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), '.'),
                        TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), '#'),
                    };

                    let color = if self.visible_tiles[idx] { visible_color } else { visible_color.to_greyscale() };

                    ctx.set(
                        x,
                        y,
                        color,
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437(character),
                    );
                }

                x += 1;
                if x > 79 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width;

        let directions = [
            // cardinal directions
            (-1,  0, -1, 1.0),
            ( 1,  0,  1, 1.0),
            ( 0, -1, -w, 1.0),
            ( 0,  1,  w, 1.0),

            // diagonals
            (-1, -1,-1-w, 1.45),
            ( 1, -1, 1-w, 1.45),
            (-1,  1,-1+w, 1.45),
            ( 1,  1, 1+w, 1.45)
        ];

        let valid_dirs = directions.iter().filter(|p| {
            let (dx,dy,_,_) = p;
            self.is_exit_valid(x + dx, y + dy)
        });

        valid_dirs.map(|d| {
            let (_,_,d_index,weight) = d;
            ((idx as i32 + d_index) as usize, *weight as f32)
        }).collect()
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
