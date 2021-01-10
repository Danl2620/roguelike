use super::{CombatStats, GameLog, InBackpack, Name, Player, State, Viewport};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

// ------------------------------------------------------------------------------------------------------------------ //
fn draw_inventory(world: &World, ctx: &mut Rltk, viewport: &Viewport) {
    let player_entity = world.fetch::<Entity>();
    let names = world.read_storage::<Name>();
    let backpack = world.read_storage::<InBackpack>();

    let inventory: Vec<(&InBackpack, &Name)> = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity)
        .collect();

    let count = inventory.len() as i32;

    let mut y = (viewport.map_height / 2 - count / 2) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut j = 0;
    for (_pack, name) in inventory {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        y += 1;
        j += 1;
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn draw_ui(world: &World, ctx: &mut Rltk, viewport: &Viewport, show_inventory: bool) {
    // draw border
    ctx.draw_box(
        0,
        viewport.map_height,
        viewport.map_width - 1,
        viewport.log_height - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // draw player health
    let combat_stats = world.read_storage::<CombatStats>();
    let players = world.read_storage::<Player>();
    for (index, info) in (&players, &combat_stats).join().enumerate() {
        let (_player, stats) = info;
        let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
        let vert_offset = (index * 1) as i32;
        ctx.print_color(
            12,
            viewport.map_height + vert_offset,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            viewport.map_height + vert_offset,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    // draw log
    let log = world.fetch::<GameLog>();
    let y = viewport.map_height + 2;
    for (i, s) in log.entries.iter().rev().enumerate() {
        let yoff = y + (i as i32);
        if yoff < viewport.map_height + viewport.log_height - 1 {
            ctx.print(2, yoff, s);
        }
    }

    if show_inventory {
        draw_inventory(world, ctx, viewport);
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

// ------------------------------------------------------------------------------------------------------------------ //
pub fn menu_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let world = &gs.ecs;
    let player_entity = world.fetch::<Entity>();
    let backpack = world.read_storage::<InBackpack>();
    let entities = world.entities();

    let inventory: Vec<(Entity, &InBackpack)> = (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .collect();
    let count = inventory.len() as i32;

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    ItemMenuResult::Selected(inventory[selection as usize].0)
                } else {
                    ItemMenuResult::NoResponse
                }
            }
        },
    }
}
