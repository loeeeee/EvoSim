use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn setup_gravity(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::ZERO;
}