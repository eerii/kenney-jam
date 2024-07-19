use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{tilemap::TILE_SEP, PlaySet};

// ······
// Plugin
// ······

pub struct MiscPlugin;

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            move_to.in_set(PlaySet::Animation),
        );
    }
}

// ··········
// Components
// ··········

pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Component)]
pub struct MoveTo {
    start: Vec2,
    target: Vec2,
    bump_dir: Option<Direction>,
    timer: Timer,
}

impl MoveTo {
    pub fn new(start: Vec2, target: Vec2, bump_dir: Option<Direction>) -> Self {
        Self {
            start,
            target,
            bump_dir,
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        }
    }
}

// ·······
// Systems
// ·······

fn move_to(
    mut cmd: Commands,
    time: Res<Time>,
    mut movables: Query<(Entity, &mut MoveTo, &mut Transform)>,
) {
    for (entity, mut to, mut trans) in movables.iter_mut() {
        let timer = to.timer.tick(time.delta());
        if timer.just_finished() {
            cmd.entity(entity).remove::<MoveTo>();
        }
        let t = timer.fraction();

        let pos = if let Some(dir) = &to.bump_dir {
            let offset = (t * PI).sin() * TILE_SEP;
            to.start + dir_to_vec(dir, offset)
        } else {
            to.start.lerp(to.target, t)
        };
        trans.translation = pos.extend(trans.translation.z);
    }
}

// ·······
// Helpers
// ·······

fn dir_to_vec(dir: &Direction, val: f32) -> Vec2 {
    match dir {
        Direction::North => Vec2::new(0., val),
        Direction::South => Vec2::new(0., -val),
        Direction::East => Vec2::new(val, 0.),
        Direction::West => Vec2::new(-val, 0.),
    }
}
