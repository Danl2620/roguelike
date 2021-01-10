use super::{
    range, CombatStats, GameLog, Item, Map, Player, Position, RunState, State, Viewshed,
    WantsToMelee, WantsToPickupItem,
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

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
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        let (nx, ny) = (
            range(1, pos.x + delta_x, map.size.width() - 1),
            range(1, pos.y + delta_y, map.size.height() - 1),
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
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("add target failed");
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
fn get_item(ecs: &mut World) {
    let player_pos = Position::new(&ecs.fetch::<Point>());
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if *position == player_pos {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
enum PlayerMove {
    Move(i32, i32),
    GetItem,
    RunState(RunState),
    None,
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // player movement
    match ctx.key {
        Some(key) => {
            let player_move = match key {
                VirtualKeyCode::Left => PlayerMove::Move(-1, 0),
                VirtualKeyCode::Right => PlayerMove::Move(1, 0),
                VirtualKeyCode::Up => PlayerMove::Move(0, -1),
                VirtualKeyCode::Down => PlayerMove::Move(0, 1),
                VirtualKeyCode::G => PlayerMove::GetItem,
                VirtualKeyCode::I => PlayerMove::RunState(RunState::ShowInventory),
                _ => PlayerMove::None,
            };

            match player_move {
                PlayerMove::Move(dx, dy) => {
                    try_move_player(dx, dy, gs);
                    RunState::PlayerTurn
                }
                PlayerMove::GetItem => {
                    get_item(&mut gs.ecs);
                    RunState::PlayerTurn
                }
                PlayerMove::RunState(state) => state,
                _ => RunState::AwaitingInput,
            }
        }
        _ => return RunState::AwaitingInput,
    }
}
