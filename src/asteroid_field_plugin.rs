use crate::collision_plugin::Collider;
use bevy::prelude::*;
use rand::Rng;

use crate::player_plugin::GameCamera;

pub struct AsteroidFieldPlugin;

#[derive(Resource)]
struct Chunks {
    loaded: Vec<Vec2>
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Bundle)]
struct AsteroidBundle {
    sprite: SpriteBundle,
    collider: Collider,
    asteroid: Asteroid
}

//this is much better then the previous attempt. Maybe the number of candidates can be adjusted, but we'll see
//TODO: observe
fn generate_points(amount: usize, min_size: f32, max_size: f32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Vec<(Vec2, f32)> {
    let mut rng = rand::thread_rng();
    let mut points : Vec<(Vec2, f32)> = Vec::new();
    points.push((Vec2::new(rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y)), rng.gen_range(min_size..max_size)));
    for _ in 0..amount {
        let mut candidates = Vec::new();
        for _ in 0..10 {
            candidates.push((Vec2::new(rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y)), rng.gen_range(min_size..max_size)));
        }
        let mut best: (usize, f32) = (0, 0.0);
        let max_dist: f32 = (max_x - min_x) * (max_x - min_x) + (max_y - min_y) * (max_y - min_y);
        for i in 0..candidates.len() {
            let mut least_distance: f32 = max_dist;
            for (loc, size) in points.clone() {
                let dist = Vec2::distance_squared(candidates[i].0, loc) - (size + candidates[i].1) * (size + candidates[i].1);
                if dist < least_distance {
                    least_distance = dist;
                }
            }
            if least_distance > best.1 {
                best = (i, least_distance);
            }
        }
        points.push(candidates[best.0]);
    }
    points
}

//TODO: size based on scale
fn build_asteroid_bundle(pos: Vec3, scale: Vec3, handle: Handle<Image>) -> AsteroidBundle {
    AsteroidBundle { sprite: SpriteBundle { transform: Transform { translation: pos, scale: scale, ..Default::default() }, 
                                                                    texture: handle, ..Default::default()}, 
                                            collider: Collider { pos: pos, size: Vec2::new(scale.x*2048.0/3.0, scale.y/3.0), flags: 1 },
                                            asteroid: Asteroid }
}

fn generate_asteroid_chunks(mut commands: Commands, asset_server: Res<AssetServer>, mut chunks: ResMut<Chunks>, query: Query<&Transform, With<GameCamera>>) {
    let mut rng = rand::thread_rng();
    let camera_transform = query.single();
    let temp = Vec2::new(camera_transform.translation.x / 4000.0, camera_transform.translation.y / 4000.0);
    let load = vec![Vec2::new(temp.x.floor(), temp.y.floor()), 
                                Vec2::new(temp.x.floor(), temp.y.ceil()), 
                                Vec2::new(temp.x.ceil(), temp.y.floor()), 
                                Vec2::new(temp.x.ceil(), temp.y.ceil())];
    for v in load {
        let mut exists = false;
        for i in 0..chunks.loaded.len() {
            if v == chunks.loaded[i] {
                exists = true;
                break ;
            }
        }
        if !exists {
            println!("loading chunk: ({}, {})", v.x, v.y);
            let points: Vec<(Vec2, f32)> = generate_points(100, 103.0, 205.0, v.x*4000.0-2000.0, v.x*4000.0+2000.0, v.y*4000.0-2000.0, v.y*4000.0+2000.0);
            for (point, size) in points {
                if point.length() > 400.0 {
                    let handle: Handle<Image> = asset_server.load(format!("Asteroids/Asteroid_output_{:04}.png", rng.gen_range(0..40)));
                    commands.spawn(build_asteroid_bundle(Vec3::new(point.x, point.y, 0.0), Vec3::ONE * size/2048.0, handle));
                    /* //For collider size reference:
                    commands.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(size/3.0).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::PURPLE)),
                        transform: Transform::from_translation(Vec3::new(point.x, point.y, 2.)),
                        ..default()
                    });*/
                }
            }
            chunks.loaded.push(v);
        }
    }
}

impl Plugin for AsteroidFieldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Chunks {loaded: Vec::new()})
            .add_systems(Update, generate_asteroid_chunks);
    }
}