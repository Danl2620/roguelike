use rltk::{VirtualKeyCode,Rltk,Point};
use specs::prelude::*;
use std::cmp::{max,min};
use super::{Position,Player,Viewshed,TileType,State,Map,RunState};

// ------------------------------------------------------------------------------------------------------------------ //
pub fn range<T: std::cmp::Ord>(l: T, v: T, u: T) -> T {
    min(u, max(v, l))
}

// ------------------------------------------------------------------------------------------------------------------ //
fn try_move_player(delta_x: i32, delta_y: i32, gs: &State) {
    let ecs = &gs.ecs;
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut playerposition = ecs.write_resource::<Point>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in
        (&mut players, &mut positions, &mut viewsheds).join() {
        let (nx, ny) = (
            range(0, pos.x + delta_x, gs.size.0 - 1),
            range(0, pos.y + delta_y, gs.size.1 - 1),
        );
        let destination_idx = map.xy_idx(nx, ny);
        if map.tiles[destination_idx] == TileType::Floor {
            pos.x = nx;
            pos.y = ny;
            playerposition.x = pos.x;
            playerposition.y = pos.y;
            viewshed.dirty = true;
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // player movement
    match ctx.key {
        Some(key) => {
            let (dx, dy, rs) = match key {
                VirtualKeyCode::Left => (-1, 0, RunState::Running),
                VirtualKeyCode::Right => (1, 0, RunState::Running),
                VirtualKeyCode::Up => (0, -1, RunState::Running),
                VirtualKeyCode::Down => (0, 1, RunState::Running),
                _ => (0, 0, RunState::Paused),
            };
            try_move_player(dx, dy, gs);
            return rs;
        }
        _ => { return RunState::Paused }
    }
}
