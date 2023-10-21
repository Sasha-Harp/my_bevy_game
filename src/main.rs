//mod hello_plugin;
mod player_plugin;
mod asteroid_field_plugin;
mod collision_plugin;
mod enemy_plugin;
use bevy::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(player_plugin::PlayerPlugin)
        .add_plugins(asteroid_field_plugin::AsteroidFieldPlugin)
        .add_plugins(enemy_plugin::EnemyPlugin)
        .add_plugins(collision_plugin::CollisionPlugin)
        .run();
}