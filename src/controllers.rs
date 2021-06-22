
use bevy::{
    prelude::*,
    ecs::schedule::ShouldRun
};

pub mod fps;
pub mod orbit;
pub mod unreal;

#[derive(Debug, PartialEq, Eq)]
pub enum InputBehavior {
    Enable,
    Disable,
}

fn set_default_input_behavior(
    mut command: Commands,
) {
    command.insert_resource(InputBehavior::Enable)
}

fn should_consume_input(
    consume: Res<InputBehavior>,
) -> ShouldRun {
    // If it has been explicitly set to no, we don't need to check again
    if InputBehavior::Disable == *consume {
        return ShouldRun::No;
    }
    // FIXME: this could be loaded after *some* initialization system set, 
    // and return Yes directly so there is no runtime perf cost
    ShouldRun::Yes
}