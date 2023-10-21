use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::collision_plugin::Collider;
use crate::asteroid_field_plugin::Asteroid;

pub struct PlayerPlugin;

#[derive(Component)]
struct MiningRay;

#[derive(Component)]
pub struct GameCamera {
    max_speed: f32,
    min_speed: f32,
    lerp_amount: f32,
    min_distance: f32,
}

#[derive(Component)]
pub struct PlayerCharacter {
    speed_swerve: f32,
    speed_thrust: f32,
    speed_reverse: f32,
    speed_rotation: f32,

    buildup_thrust: f32,
    buildup_swerveright: f32,
    buildup_swerveleft: f32,
    buildup_reverse: f32,

    buildup_speed: f32,
    falloff_speed: f32,
}

impl PlayerCharacter {
    fn new() -> PlayerCharacter {
        PlayerCharacter { speed_swerve: 50.0, speed_thrust: 150.0, speed_reverse: 30.0, speed_rotation: 1.0, 
                        buildup_thrust: 0.0, buildup_swerveright: 0.0, buildup_swerveleft: 0.0, buildup_reverse: 0.0, 
                        buildup_speed: 0.75, falloff_speed: 1.5 }
    }
}

fn interpolate(a: Vec3, b: Vec3, amount: f32) -> Vec3 {
    //assert!(amount >= 0.0 && amount <= 1.0, "expected amount between 0 and 1, got {}", amount);
    a + (b - a) * amount
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera {max_speed: 500.0, min_speed: 50.0, lerp_amount: 1.0, min_distance: 10.0}));
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((SpriteBundle {transform: Transform { scale: Vec3 { x: 0.07, y: 0.07, z: 1.0 }, translation: Vec3 { x: 0.0, y: 0.0, z: 1.0 }, ..Default::default() }, texture: asset_server.load("PC.png"), ..Default::default()}, 
                            PlayerCharacter::new(), 
                            Collider{pos: Vec3::ZERO, size: Vec2::new(0.0, 0.0), flags: 1}));
}

fn control_player(input: Res<Input<KeyCode>>, time:Res<Time>, mut camera_query: Query<(&mut Transform, &GameCamera), Without<PlayerCharacter>>, mut player_query: Query<(&mut Transform, &mut PlayerCharacter), Without<GameCamera>>) {
    //unpack queries
    let (mut player_transform, mut player) = player_query.single_mut();
    let (mut camera_transform, camera) = camera_query.single_mut();

    //translate the player
    let rotated_x: Vec3 = Quat::mul_vec3(player_transform.rotation, Vec3::X);
    let rotated_y: Vec3 = Quat::mul_vec3(player_transform.rotation, Vec3::Y);
    if input.pressed(KeyCode::D) {
        player.buildup_swerveright = f32::min(1.0, player.buildup_swerveright + time.delta_seconds() * player.buildup_speed);
    } else {
        player.buildup_swerveright = f32::max(0.0, player.buildup_swerveright - time.delta_seconds() * player.falloff_speed);
    }
    if input.pressed(KeyCode::A) {
        player.buildup_swerveleft = f32::min(1.0, player.buildup_swerveleft + time.delta_seconds() * player.buildup_speed);
    } else {
        player.buildup_swerveleft = f32::max(0.0, player.buildup_swerveleft - time.delta_seconds() * player.falloff_speed);
    }
    if input.pressed(KeyCode::S) {
        player.buildup_reverse = f32::min(1.0, player.buildup_reverse + time.delta_seconds() * player.buildup_speed);
    } else {
        player.buildup_reverse = f32::max(0.0, player.buildup_reverse - time.delta_seconds() * player.falloff_speed);
    }
    if input.pressed(KeyCode::W) {
        player.buildup_thrust = f32::min(1.0, player.buildup_thrust + time.delta_seconds() * player.buildup_speed);
    } else {
        player.buildup_thrust = f32::max(0.0, player.buildup_thrust - time.delta_seconds() * player.falloff_speed);
    }
    player_transform.translation += rotated_x * player.speed_swerve * time.delta_seconds() * player.buildup_swerveright;
    player_transform.translation += -rotated_x * player.speed_swerve * time.delta_seconds() * player.buildup_swerveleft;
    player_transform.translation += -rotated_y * player.speed_reverse * time.delta_seconds() * player.buildup_reverse;
    player_transform.translation += rotated_y * player.speed_thrust * time.delta_seconds() * player.buildup_thrust;

    //rotate the player
    if input.pressed(KeyCode::Q) {
        player_transform.rotate_z(player.speed_rotation * time.delta_seconds());
    }
    if input.pressed(KeyCode::E) {
        player_transform.rotate_z(-1.0 * player.speed_rotation * time.delta_seconds());
    }
    
    //translate the camera by lerping from the camera position to the player position
    let target: Vec3 = interpolate(camera_transform.translation, player_transform.translation, camera.lerp_amount * time.delta_seconds());
    if Vec3::distance(camera_transform.translation, player_transform.translation) < camera.min_distance {
        return ;
    }
    if Vec3::distance(camera_transform.translation, player_transform.translation) < camera.min_speed * time.delta_seconds() {
        info!("camera snap");
        camera_transform.translation = player_transform.translation;
    } else if Vec3::distance(camera_transform.translation, target) < camera.min_speed * time.delta_seconds() {
        let minimum_path: Vec3 = Vec3::normalize(player_transform.translation - camera_transform.translation) * camera.min_speed * time.delta_seconds();
        camera_transform.translation += minimum_path;
    } else if Vec3::distance(camera_transform.translation, target) > camera.max_speed * time.delta_seconds() {
        let maximum_path: Vec3 = Vec3::normalize(player_transform.translation - camera_transform.translation) * camera.max_speed * time.delta_seconds();
        camera_transform.translation += maximum_path;
    } else {
        camera_transform.translation = target;
    }

    
}

fn control_mining_laser(mut commands: Commands, mouse: Res<Input<MouseButton>>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>, mut player_query: Query<(&Transform, &mut PlayerCharacter), Without<Asteroid>>, asteroid_query: Query<(&Transform, &Collider, &Asteroid), (Without<PlayerCharacter>, Without<MiningRay>)>, mut ray_query: Query<(Entity, &mut Transform), (With<MiningRay>, Without<PlayerCharacter>, Without<Asteroid>)>) {
    //mining
    let (player_transform, _player_character) = player_query.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        commands.spawn((SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.125, 0.25),
                custom_size: Some(Vec2::new(5.0, 500.0)),
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation + player_transform.rotation.mul_vec3(Vec3{x: 15.0, y:250.0, z:-0.5}))
                .with_rotation(player_transform.rotation),
            ..default()}, MiningRay));
        commands.spawn((SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.125, 0.25),
                custom_size: Some(Vec2::new(5.0, 500.0)),
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation + player_transform.rotation.mul_vec3(Vec3{x: -15.0, y:250.0, z:-0.5}))
                .with_rotation(player_transform.rotation),
            ..default()}, MiningRay));
    }
    if mouse.just_released(MouseButton::Left) {
        for (e, _) in ray_query.iter() {
            commands.entity(e).despawn();
        }
    } else {
        let mut i = 1.0;
        for (_, mut transform) in ray_query.iter_mut() {
            //calculations for raycasting (just 2d line intersections with circles, vie distance from center + check if end of line is in circle)
            let start_point = player_transform.translation + player_transform.rotation.mul_vec3(Vec3{x: i * 15.0, y:250.0, z:-0.5});
            let player_point = player_transform.translation + player_transform.rotation.mul_vec3(Vec3{x: i * 15.0, y:0.0, z:-0.5});
            let direction = player_transform.rotation.mul_vec3(Vec3::Y);
            let end_point = player_transform.translation + player_transform.rotation.mul_vec3(Vec3{x: i * 15.0, y:0.0, z:-0.5}) + direction * 500.0;
            let normal = Quat::from_rotation_z(0.5 * PI).mul_vec3(direction);
            let divisor = 1.0 / (normal.x * direction.y - direction.x * normal.y);
            let partial_multiplier = player_point.x * direction.y - player_point.y * direction.x;
            //draw lasers correctly
            transform.translation = start_point;
            transform.rotation = player_transform.rotation;
            i = -1.0;
            //raycasting to all asteroids (should somehow decrease num of asteroids, but it's fine for now)
            //This was WAY too difficult, because I was WAY too stupid
            //TODO: actual collision + better endpoint.distance flow control
            for (asteroid_transform, collider, _asteroid) in asteroid_query.iter() {
                let x = (partial_multiplier + asteroid_transform.translation.y * direction.x - asteroid_transform.translation.x * direction.y) * divisor;
                if end_point.distance(asteroid_transform.translation) < collider.size.x {
                    commands.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(10.0).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::PURPLE)),
                        transform: Transform::from_translation(Vec3::new(end_point.x, end_point.y, 2.)),
                        ..default()
                    });
                } else if x.abs() < collider.size.x {
                    let intersection = asteroid_transform.translation + x * normal;
                    let y = if direction.x != 0.0 {(intersection.x - player_point.x) / direction.x} else {(intersection.y - player_point.y) / direction.y};
                    if y > 0.0 && y < 500.0 {
                        commands.spawn(MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Circle::new(10.0).into()).into(),
                            material: materials.add(ColorMaterial::from(Color::PURPLE)),
                            transform: Transform::from_translation(Vec3::new(intersection.x, intersection.y, 2.)),
                            ..default()
                        });
                    }
                }
            }
        }
    }

}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_player))
            .add_systems(Update, (control_player, control_mining_laser));
    }
}