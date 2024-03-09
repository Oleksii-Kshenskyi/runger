use std::{error::Error, fmt::Display};

use bevy::{prelude::*, sprite::Mesh2dHandle};

use crate::simulation::players::FacingDirection;

use super::config::default_entity_size;

#[derive(Debug)]
pub struct RungerError {
    message: String,
}

pub fn rerror(msg: &'static str) -> Box<dyn Error> {
    Box::new(RungerError {
        message: msg.to_string(),
    })
}

impl Display for RungerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RungerError {}

pub fn triangle_facing(
    direction: &FacingDirection,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Mesh2dHandle {
    match *direction {
        FacingDirection::Up => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(0., default_entity_size() / 2.),
            Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
        FacingDirection::Right => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., 0.),
            Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
        FacingDirection::Down => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(0., -default_entity_size() / 2.),
        ))),
        FacingDirection::Left => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., 0.),
            Vec2::new(default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
    }
}
