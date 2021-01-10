use specs::prelude::*;
use specs_derive::*;
use std::cmp::{max,min,Ordering};
use std::ops::{Add, Sub};
use rltk::{RGB,Point};

// ------------------------------------------------------------------------------------------------------------------ //
pub fn range<T: std::cmp::Ord>(l: T, v: T, u: T) -> T {
    min(u, max(v, l))
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Default, Component, Copy, Clone, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

pub fn manhattan_dist(p: &Position, q: &Position) -> i32 {
    (q.x - p.x) + (q.y - q.x)
}

impl Position {
    pub fn new(p: &Point) -> Position {
        Position { x: p.x, y: p.y }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {x: self.x - other.x, y: self.y - other.y}
    }
}

impl Eq for Position {}
impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        let origin = Position { x: 0, y: 0};
        manhattan_dist(&origin, self)
            .partial_cmp(&manhattan_dist(&origin, other))
            .unwrap_or(Ordering::Equal)
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("unable to insert damage");
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct BlocksTile {}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Player {}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Monster {}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Item {}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }
