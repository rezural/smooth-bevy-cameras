use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};
use super::{set_default_input_behavior, should_consume_input};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(set_default_input_behavior.system())
            .add_system(control_system.system())
            .add_event::<ControlEvent>()
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(should_consume_input.system())
                    .with_system(default_input_map.system())
            );
    }
}

#[derive(Bundle)]
pub struct OrbitCameraBundle {
    controller: OrbitCameraController,
    #[bundle]
    look_transform: LookTransformBundle,
    #[bundle]
    perspective: PerspectiveCameraBundle,
}

impl OrbitCameraBundle {
    pub fn new(
        controller: OrbitCameraController,
        mut perspective: PerspectiveCameraBundle,
        eye: Vec3,
        target: Vec3,
    ) -> Self {
        // Make sure the transform is consistent with the controller to start.
        perspective.transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);

        Self {
            controller,
            look_transform: LookTransformBundle {
                transform: LookTransform { eye, target },
                smoother: Smoother::new(controller.smoothing_weight),
            },
            perspective,
        }
    }
}

/// A 3rd person camera that orbits around the target.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct OrbitCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub mouse_translate_sensitivity: Vec2,
    pub mouse_wheel_zoom_sensitivity: f32,
    pub smoothing_weight: f32,
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self {
            mouse_rotate_sensitivity: Vec2::splat(0.006),
            mouse_translate_sensitivity: Vec2::splat(0.008),
            mouse_wheel_zoom_sensitivity: 0.15,
            smoothing_weight: 0.8,
            enabled: true,
        }
    }
}

pub enum ControlEvent {
    Orbit(Vec2),
    TranslateTarget(Vec2),
    Zoom(f32),
}

pub struct DisableDefaultInput;

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().next() {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        enabled,
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        ..
    } = *controller;

    if !enabled {
        return;
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if keyboard.pressed(KeyCode::LControl) {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        scalar *= 1.0 + event.y * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform)>,
) {
    // Can only control one camera at a time.
    let (controller, mut transform, scene_transform) =
        if let Some((controller, transform, scene_transform)) = cameras.iter_mut().next() {
            (controller, transform, scene_transform)
        } else {
            return;
        };

    if controller.enabled {
        let mut look_angles = LookAngles::from_vector(-transform.look_direction());
        let mut radius_scalar = 1.0;

        for event in events.iter() {
            match event {
                ControlEvent::Orbit(delta) => {
                    look_angles.add_yaw(-delta.x);
                    look_angles.add_pitch(delta.y);
                }
                ControlEvent::TranslateTarget(delta) => {
                    let right_dir = scene_transform.rotation * -Vec3::X;
                    let up_dir = scene_transform.rotation * Vec3::Y;
                    transform.target += delta.x * right_dir + delta.y * up_dir;
                }
                ControlEvent::Zoom(scalar) => {
                    radius_scalar *= scalar;
                }
            }
        }

        look_angles.assert_not_looking_up();

        transform.eye =
            transform.target + radius_scalar * transform.radius() * look_angles.unit_vector();
    } else {
        events.iter(); // Drop the events.
    }
}
