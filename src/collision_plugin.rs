use bevy::prelude::*;

pub struct CollisionPlugin;

#[derive(Component)]
pub struct Collider {
    pub pos: Vec3,
    pub size: Vec2,
    pub flags: u32
}

fn handle_collisions(time: Res<Time>, query: Query<(&Transform, &mut Collider)>) {
    
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_collisions);
    }
}