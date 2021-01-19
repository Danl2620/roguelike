use super::{spawner, Position, Rect, Viewport, Viewshed};
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

mod map_utils {
    use super::Rect;
    use super::TileType;
    use std::cmp::{max, min};

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn apply_horizontal_tunnel(
        map: &Rect,
        x1: i32,
        x2: i32,
        y: i32,
        tiles: &mut Vec<TileType>,
    ) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.area() as usize {
                tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn apply_vertical_tunnel(map: &Rect, y1: i32, y2: i32, x: i32, tiles: &mut Vec<TileType>) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.area() as usize {
                tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn apply_room_to_map(map: &Rect, room: &Rect, tiles: &mut Vec<TileType>) {
        for y in room.min.y..room.max.y {
            for x in room.min.x..room.max.x {
                let idx = map.xy_idx(x, y);
                tiles[idx] = TileType::Floor;
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Default)]
pub struct Map {
    pub size: Rect,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        self.size.xy_idx(x, y)
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn position_idx(&self, p: Position) -> usize {
        self.xy_idx(p.x, p.y)
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.size.contains(&Position { x, y }) {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
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
    pub fn new_map_rooms_and_corridors(
        world: &mut World,
        viewport: &Viewport,
        rng: &mut RandomNumberGenerator,
    ) -> Map {
        let size = Rect::new(0, 0, viewport.map_width, viewport.map_height);
        let mut tiles = vec![TileType::Wall; size.area() as usize];
        let mut rooms = Vec::new();

        const MAX_ROOMS: i32 = 30;
        const SIZE_RANGE: (i32, i32) = (6, 10);

        for _ in 0..MAX_ROOMS {
            let (w, h) = (
                rng.range(SIZE_RANGE.0, SIZE_RANGE.1),
                rng.range(SIZE_RANGE.0, SIZE_RANGE.1),
            );
            let (x, y) = (
                rng.range(1, size.width() - w - 1),
                rng.range(1, size.height() - h - 1),
            );
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                    break;
                }
            }
            if ok {
                rooms.push(new_room);
            }
        }

        // some test rooms instead:
        // rooms.push(Rect::new(2, 1, 6, 6));
        // rooms.push(Rect::new(1, 12, 6, 6));

        dbg!(for room in &rooms {
            room.print_debug();
        });

        for (index, room) in rooms.iter().enumerate() {
            map_utils::apply_room_to_map(&size, &room, &mut tiles);

            if index > 0 {
                let (new_x, new_y) = room.center();
                let (prev_x, prev_y) = rooms[index - 1].center();
                let (tunnel_x, tunnel_y) = if rng.range(0, 2) == 1 {
                    (new_x, prev_y)
                } else {
                    (prev_x, new_y)
                };
                map_utils::apply_vertical_tunnel(&size, prev_y, new_y, tunnel_x, &mut tiles);
                map_utils::apply_horizontal_tunnel(&size, prev_x, new_x, tunnel_y, &mut tiles);
            }

            spawner::spawn_room(world, rng, &room);
        }

        let vec_size = size.area() as usize;
        Map {
            size,
            tiles,
            rooms,
            revealed_tiles: vec![false; vec_size],
            visible_tiles: vec![false; vec_size],
            blocked: vec![false; vec_size],
            tile_content: vec![Vec::new(); vec_size],
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn draw_map_visibility(&self, ecs: &World, viewport: &Viewport, ctx: &mut Rltk) {
        let mut viewsheds = ecs.write_storage::<Viewshed>();

        let floor = rltk::to_cp437('.');
        let wall = rltk::to_cp437('#');
        //let numbers = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let black = RGB::from_f32(0., 0., 0.);

        for _viewshed in (&mut viewsheds).join() {
            let mut y = 0 as i32;
            let mut x = 0 as i32;
            for (idx, tile) in self.tiles.iter().enumerate() {
                //render a tile depending on the tile type

                if self.revealed_tiles[idx] {
                    let (visible_color, character) = match tile {
                        TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), floor),
                        TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), wall),
                        // TileType::Wall => (
                        //     RGB::from_f32(0.0, 1.0, 0.0),
                        //     rltk::to_cp437(numbers[(x % 10) as usize]),
                        // ),
                    };

                    let color = if self.visible_tiles[idx] {
                        visible_color
                    } else {
                        visible_color.to_greyscale()
                    };

                    ctx.set(x, y, color, black, character);
                }

                x += 1;
                if x > viewport.map_width - 1 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn draw_map_debug(&self, ecs: &World, viewport: &Viewport, ctx: &mut Rltk) {
        let floor = rltk::to_cp437('.');
        let wall = rltk::to_cp437('#');
        //let numbers = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let black = RGB::from_f32(0., 0., 0.);

        let mut y = 0;
        let mut x = 0;
        for tile in self.tiles.iter() {
            // Render a tile depending upon the tile type
            let (color, character) = match tile {
                TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), floor),
                TileType::Wall => (
                    RGB::from_f32(0.0, 1.0, 0.0),
                    //rltk::to_cp437(numbers[(x % 10) as usize]),
                    wall,
                ),
            };

            ctx.set(x, y, color, black, character);

            // Move the coordinates
            x += 1;
            if x > viewport.map_width - 1 {
                x = 0;
                y += 1;
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn draw_map(&self, ecs: &World, viewport: &Viewport, ctx: &mut Rltk) {
        self.draw_map_visibility(ecs, viewport, ctx)
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.size.width(), self.size.height())
    }

    fn in_bounds(&self, pos: rltk::Point) -> bool {
        let bounds = self.dimensions();
        pos.x >= 0 && pos.x < bounds.x && pos.y >= 0 && pos.y < bounds.y
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let position = self.size.idx_position(idx);
        let w = self.size.width();

        let directions = [
            // cardinal directions
            (-1, 0, -1, 1.0),
            (1, 0, 1, 1.0),
            (0, -1, -w, 1.0),
            (0, 1, w, 1.0),
            // diagonals
            (-1, -1, -1 - w, 1.45),
            (1, -1, 1 - w, 1.45),
            (-1, 1, -1 + w, 1.45),
            (1, 1, 1 + w, 1.45),
        ];

        let valid_dirs = directions.iter().filter(|p| {
            let (dx, dy, _, _) = p;
            self.is_exit_valid(position.x + dx, position.y + dy)
        });

        valid_dirs
            .map(|d| {
                let (_, _, d_index, weight) = d;
                ((idx as i32 + d_index) as usize, *weight as f32)
            })
            .collect()
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.size.width() as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
