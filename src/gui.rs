use rltk::{RGB, Rltk, Console};
use specs::prelude::*;
use super::{Viewport,CombatStats,Player,GameLog};

// ------------------------------------------------------------------------------------------------------------------ //
pub fn draw_ui(ecs: &World, ctx: &mut Rltk, viewport: &Viewport) {
    // draw border
    ctx.draw_box(
        0,
        viewport.map_height,
        viewport.map_width-1,
        viewport.log_height-1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK));

    // draw player health
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (index, info) in (&players, &combat_stats).join().enumerate() {
        let (_player, stats) = info;
        let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
        let vert_offset = (index*1) as i32;
        ctx.print_color(
            12, 
            viewport.map_height + vert_offset,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health);

        ctx.draw_bar_horizontal(
            28,
            viewport.map_height + vert_offset,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK));
    }

    // draw log
    let log = ecs.fetch::<GameLog>();
    let mut y = viewport.map_height + 2;
    for (i,s) in log.entries.iter().rev().enumerate() {
        let yoff = y + (i as i32);
        if yoff < viewport.map_height + viewport.log_height - 1 {
            ctx.print(2, yoff, s);
        }
    }

}