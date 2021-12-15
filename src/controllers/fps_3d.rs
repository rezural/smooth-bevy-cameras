use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{mouse::MouseMotion, prelude::*},
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Fps3dCameraPlugin {
    pub override_input_system: bool,
}

impl Fps3dCameraPlugin {
    pub fn new(override_input_system: bool) -> Self {
        Self {
            override_input_system,
        }
    }
}

impl Plugin for Fps3dCameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let app = app
            .add_system(control_system.system())
            .add_event::<ControlEvent>();
        if !self.override_input_system {
            app.add_system(default_input_map.system());
        }
    }
}

#[derive(Bundle)]
pub struct Fps3dCameraBundle {
    controller: Fps3dCameraController,
    #[bundle]
    look_transform: LookTransformBundle,
    #[bundle]
    perspective: PerspectiveCameraBundle,
}

impl Fps3dCameraBundle {
    pub fn new(
        controller: Fps3dCameraController,
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

/// Your typical first-person camera controller.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Fps3dCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub translate_sensitivity: f32,
    pub smoothing_weight: f32,
}

impl Default for Fps3dCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_rotate_sensitivity: Vec2::splat(0.002),
            translate_sensitivity: 0.5,
            smoothing_weight: 0.9,
        }
    }
}

pub enum ControlEvent {
    Rotate(Vec2),
    TranslateEye(Vec3),
}

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    cameras: Query<(&Fps3dCameraController, &LookTransform)>,
) {
    // Can only control one camera at a time.
    let (controller, transform) = if let Some((controller, transform)) = cameras.iter().next() {
        (controller, transform)
    } else {
        return;
    };

    let Fps3dCameraController {
        enabled,
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    if !enabled {
        return;
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    events.send(ControlEvent::Rotate(
        mouse_rotate_sensitivity * cursor_delta,
    ));

    let look_vector = transform.look_direction();

    for (key, dir) in [
        (KeyCode::W, look_vector),
        (KeyCode::A, -look_vector.cross(Vec3::Y)),
        (KeyCode::S, -look_vector),
        (KeyCode::D, look_vector.cross(Vec3::Y)),
    ]
    .iter()
    .cloned()
    {
        if keyboard.pressed(key) {
            events.send(ControlEvent::TranslateEye(translate_sensitivity * dir));
        }
    }
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&Fps3dCameraController, &mut LookTransform)>,
) {
    // Can only control one camera at a time.
    let (controller, mut transform) =
        if let Some((controller, transform)) = cameras.iter_mut().next() {
            (controller, transform)
        } else {
            return;
        };

    if controller.enabled {
        let look_vector = transform.look_direction();
        let mut look_angles = LookAngles::from_vector(look_vector);

        for event in events.iter() {
            match event {
                ControlEvent::Rotate(delta) => {
                    // Rotates with pitch and yaw.
                    look_angles.add_yaw(-delta.x);
                    look_angles.add_pitch(-delta.y);
                }
                ControlEvent::TranslateEye(delta) => {
                    // Translates up/down (Y) left/right (X) and forward/back (Z).
                    transform.eye += *delta;
                }
            }
        }

        look_angles.assert_not_looking_up();

        transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
    } else {
        events.iter(); // Drop the events.
    }
}
