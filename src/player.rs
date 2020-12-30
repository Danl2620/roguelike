use rltk::{VirtualKeyCode,Rltk,Point};
use specs::prelude::*;
use super::{CombatStats,Position,Player,Viewshed,State,Map,RunState,range,WantsToMelee};

// ------------------------------------------------------------------------------------------------------------------ //
fn try_move_player(delta_x: i32, delta_y: i32, gs: &State) {
    let ecs = &gs.ecs;
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut playerposition = ecs.write_resource::<Point>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    // let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join() {
        let (nx, ny) = (
            range(0, pos.x + delta_x, gs.viewport.map_width - 1),
            range(0, pos.y + delta_y, gs.viewport.map_height - 1),
        );
        let destination_idx = map.xy_idx(nx, ny);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            // let name = names.get(*potential_target);
            if let Some(_t) = target {
                // found a target, attack it!
                // if let Some(n) = name {
                //     console::log(&format!("From hell's heart, I stab at thee ({})!", &n.name));
                // }
                wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target}).expect("add target failed");
                return; // @#@
            }
        }

        // no target found, try moving
        if !map.blocked[destination_idx] {
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
                VirtualKeyCode::Left => (-1, 0, RunState::PlayerTurn),
                VirtualKeyCode::Right => (1, 0, RunState::PlayerTurn),
                VirtualKeyCode::Up => (0, -1, RunState::PlayerTurn),
                VirtualKeyCode::Down => (0, 1, RunState::PlayerTurn),
                _ => (0, 0, RunState::AwaitingInput),
            };
            try_move_player(dx, dy, gs);
            return rs;
        }
        _ => { return RunState::AwaitingInput }
    }
}
