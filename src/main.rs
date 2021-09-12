// #![allow(unused)]

mod player;
mod enemy;
use bevy::prelude::*;
use crate::{enemy::EnemyPlugin, player::PlayerPlugin};
use bevy::sprite::collide_aabb::collide;

const PLAYER_SPRITE: &str = "player_a_01.png";
const LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const TIME_STEP: f32 = 1. / 60.;
const SCALE: f32 = 0.5;

pub struct Materials{
    player: Handle<ColorMaterial>,
    laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
}
struct WinSize{
    #[allow(unused)]
    w: f32,
    h: f32,
}
struct ActiveEnemies(u32);

struct Player;
struct PlayerReadyFire(bool);
struct Laser;

struct Enemy;

struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500.)
    }
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
        .insert_resource(WindowDescriptor{
            title: "Rust Invaders".to_string(),
            width: 598.,
            height: 676.,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup.system())
        .add_system(laser_hit_enemy.system())
        .run();
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut materials: ResMut<Assets<ColorMaterial>>,
         mut windows: ResMut<Windows>
){
    let mut window = windows.get_primary_mut().unwrap();
    //camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    //create main resources
    commands.insert_resource(Materials{
        player: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        laser: materials.add(asset_server.load(LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
    });
    commands.insert_resource(WinSize{w: window.height(), h: window.width()});

    //window

    //spawn a sprite
}

fn laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform, &Sprite, With<Laser>)>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>,
){
    for(laser_entity, laser_tf, laser_sprite, _) in laser_query.iter_mut(){
        for(enemy_entity, enemy_tf, enemy_sprite, _) in enemy_query.iter_mut(){
            let laser_scale = Vec2::from(laser_tf.scale);
            let enemy_scale = Vec2::from(enemy_tf.scale);
            let collision = collide(
                laser_tf.translation,
                laser_sprite.size * laser_scale,
                enemy_tf.translation,
                enemy_sprite.size * enemy_scale,
            );

            if let Some(_) = collision {
                // remove the enemy
                commands.entity(enemy_entity).despawn();
                active_enemies.0 -= 1;

                // remove the laser
                commands.entity(laser_entity).despawn();

            }
        }
    }
}

