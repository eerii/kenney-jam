use std::{f32::consts::PI, slice::Iter};

use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::{tilemap::TILE_SEP, GameState};

pub const MIN_TURN_TIMER: f32 = 0.2;

// ······
// Plugin
// ······

pub struct MiscPlugin;

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            move_to.run_if(in_state(GameState::Play)),
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

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        }
    }
}

impl Direction {
    pub fn iter() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        DIRECTIONS.iter()
    }
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
            timer: Timer::from_seconds(MIN_TURN_TIMER, TimerMode::Once),
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
        let t = (timer.fraction() * 1.3).clamp(0., 1.);

        let pos = if let Some(dir) = &to.bump_dir {
            let offset = (t * PI).sin() * TILE_SEP;
            to.start + dir_to_vec(dir, offset)
        } else {
            to.start.lerp(to.target, t)
        };
        trans.translation = pos.extend(trans.translation.z);
        trans.rotation = Quat::from_rotation_arc(
            Vec3::Y,
            Vec3::new((t * 2. * PI).sin() * 0.05, 0., 0.),
        );
    }
}

// ·······
// Helpers
// ·······

pub fn dir_to_vec(dir: &Direction, val: f32) -> Vec2 {
    match dir {
        Direction::North => Vec2::new(0., val),
        Direction::South => Vec2::new(0., -val),
        Direction::East => Vec2::new(val, 0.),
        Direction::West => Vec2::new(-val, 0.),
    }
}
