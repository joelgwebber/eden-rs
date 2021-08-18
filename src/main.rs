use std::fs;

use crate::kurt::expr::{_app, _id, _qid};
use crate::kurt::Kurt;

mod kurt;

#[macro_use]
extern crate pest_derive;

fn main() {
    const MAIN: &str = "./src/eden.kurt";
    let src = fs::read_to_string(MAIN).expect("cannot read file");

    let kurt = Kurt::new();
    kurt.eval_src(MAIN, src.as_str());
    kurt.eval(
        &kurt.root,
        &_app(vec![_app(vec![_id("World"), _qid("init")])]),
    );

    // App::build()
    //     .insert_resource(Msaa { samples: 4 })
    //     .insert_resource(WindowDescriptor {
    //         title: "Eden".into(),
    //         width: 1600.,
    //         height: 1200.,
    //         ..Default::default()
    //     })
    //     .add_plugins(DefaultPlugins)
    //     .add_system(rotator_system.system())
    //     .add_startup_system(setup.system())
    //     .run();
}

/*
// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // object
    commands.spawn_scene(asset_server.load("models/bed_out/bed.gltf#Scene0"));

    // light
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(4.0, 4.0, 4.0),
            ..Default::default()
        })
        .insert(Rotates);

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 4.0, 4.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..Default::default()
    });
}

struct Rotates;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_y(
            (4.0 * std::f32::consts::PI / 20.0) * time.delta_seconds(),
        )) * *transform;
    }
}
*/
