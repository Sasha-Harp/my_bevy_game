use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;
use crate::player_plugin::PlayerCharacter;
use crate::collision_plugin::Collider;

pub struct EnemyPlugin;

const DEPTH_MOD: f32 = 2000000.0;
const BASE_SPAWNCHANCE: f32 = 0.1;
const TARGET_DIST: f32 = 400.0;
const TARGET_DIST_SQ: f32 = TARGET_DIST * TARGET_DIST;
const ENEMY_SPEED: f32 = 200.0;
const DIVE_CHANCE: f32 = 0.03;
const DIVE_RANGE: f32 = 50.0;

#[derive(Component)]
struct Enemy {
    is_diving: bool,
    dive_point: Vec3
}

impl Enemy {
    fn new() -> Enemy {
        Enemy {is_diving: false, dive_point: Vec3::ZERO}
    }
}

#[derive(Bundle)]
struct EnemyBundle {
    sprite: SpriteBundle,
    enemy: Enemy,
    collider: Collider
}

fn spawn_enemies(mut commands: Commands, time: Res<Time>, asset_server: Res<AssetServer>, query: Query<&Transform, With<PlayerCharacter>>) {
    let mut rng = rand::thread_rng();
    let player_position: Vec3 = query.single().translation;
    let player_depth: f32 = (player_position.x.powi(2) + player_position.y.powi(2)) / DEPTH_MOD;
    let spawn_probability: f32 = 1.0 - (1.0 - player_depth * BASE_SPAWNCHANCE).powf(time.delta_seconds());
    if rng.gen_range(0.0..1.0) < spawn_probability {
        let enemy_position: Vec3 = Quat::from_rotation_z(rng.gen_range(0.0..(2.0*PI))).mul_vec3(Vec3::ONE) * 500.0 + player_position;
        commands.spawn(EnemyBundle { sprite: SpriteBundle {transform: Transform { scale: Vec3 { x: 0.05, y: 0.05, z: 1.0 }, translation: enemy_position, ..Default::default() }, texture: asset_server.load("enemy.png"), ..Default::default()}, 
                            enemy: Enemy::new(), 
                            collider: Collider{pos: Vec3::ZERO, size: Vec2::new(0.0, 0.0), flags: 1}});
    }
}

fn control_enemies(time: Res<Time>, mut query_enemies: Query<(&mut Transform, &mut Enemy), Without<PlayerCharacter>>, query_player: Query<&Transform, (With<PlayerCharacter>, Without<Enemy>)>) {
    let player_position: Vec3 = query_player.single().translation;
    let rotation: Quat = Quat::from_rotation_z(PI/2.0);
    query_enemies.for_each_mut(|(mut transform, enemy)| {
        let position: Vec3 = transform.translation;
        let player_direction: Vec3 = Vec3::normalize_or_zero(position - player_position);
        if !enemy.is_diving {
            let distance: f32 = position.distance(player_position);
            let direction1: Vec3 = ((player_direction * TARGET_DIST + player_position) - position) * 0.0003;
            let direction2: Vec3 = Vec3::normalize_or_zero(rotation.mul_vec3(player_position - position)) / (distance - TARGET_DIST).abs().max(1.0);
            transform.translation += (direction1 + direction2).normalize_or_zero() * ENEMY_SPEED * time.delta_seconds();
        } else {
            transform.translation += (enemy.dive_point + player_position - position).normalize_or_zero() * ENEMY_SPEED * 2.0 * time.delta_seconds();
        }
        transform.rotation = Quat::from_rotation_arc_2d(Vec2::new(0.0, -1.0), Vec2::new(player_direction.x, player_direction.y));
    });

    for (transform, mut enemy) in query_enemies.iter_mut().filter(|(transform, enemy)| enemy.is_diving || Vec3::distance_squared(player_position, transform.translation)-TARGET_DIST_SQ < 100.0) {
        let mut rng = rand::thread_rng();
        let dive_probability: f32 = 1.0 - (1.0 - DIVE_CHANCE).powf(time.delta_seconds());
        if !enemy.is_diving {
            if rng.gen_range(0.0..1.0) < dive_probability {
                enemy.is_diving = true;
                enemy.dive_point = Quat::from_rotation_z(rng.gen_range(0.0..(2.0*PI))).mul_vec3(Vec3::X) * DIVE_RANGE;
            }
        }
        if enemy.is_diving {
            enemy.is_diving = Vec3::distance(transform.translation, player_position + enemy.dive_point) >= 10.0
        }
    }
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_enemies)
            .add_systems(Update, control_enemies);
    }
}