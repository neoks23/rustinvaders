// #![allow(unused)]

mod player;
mod enemy;
mod window;
mod ui;
mod state;

use bevy::prelude::*;
use crate::{enemy::EnemyPlugin, player::PlayerPlugin};
use bevy::sprite::collide_aabb::collide;
use std::collections::HashSet;
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin, InspectableRegistry};
use bevy_inspector_egui::widgets::{InspectorQuerySingle, InspectorQuery, ResourceInspector};
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy::ui::widget::Image;
use sqlx::mysql::{MySqlPoolOptions, MySqlPool};
use futures::executor::block_on;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_egui::egui::CtxRef;
use crate::window::WindowPlugin;
use crate::ui::UiPlugin;
use crate::state::StatePlugin;

const PLAYER_SPRITE: &str = "player_c_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_SPRITE: &str = "enemy_b_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const FIRING_SFX: &str = "Audio/Galaga_Firing_Sound_Effect.mp3";
const KILL_SFX: &str = "Audio/Galaga_Kill_Enemy_Sound_Effect.mp3";
const DEAD_SFX: &str = "Audio/m01se_03hit1.mp3";
const GAMEOVER_SFX: &str = "Audio/GALAGA_NAME_ENTRY_MUSIC_ARRANGE_VERSION.mp3";
const TIME_STEP: f32 = 1. / 60.;
const SCALE: f32 = 0.5;
const MAX_ENEMIES: u32 = 4;
const MAX_FORMATION_MEMBERS: u32 = 2;
const PLAYER_RESPAWN_DELAY: f64 = 2.;
const BEVY_TEXTURE_ID: u64 = 0;

pub struct Materials{
    player: Handle<ColorMaterial>,
    player_laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    enemy_laser: Handle<ColorMaterial>,
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
    explosion: Option<Handle<TextureAtlas>>,
}

impl FromWorld for Materials{
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let asset_server = asset_server.clone();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        let texture_handle = asset_server.load(EXPLOSION_SHEET);
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0,64.0), 4, 4);
        let mut material = Materials{
            player: materials.add(asset_server.load(PLAYER_SPRITE).into()),
            player_laser: materials.add(asset_server.load(PLAYER_LASER_SPRITE).into()),
            enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
            enemy_laser: materials.add(asset_server.load(ENEMY_LASER_SPRITE).into()),
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            explosion: None,
        };

        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        material.explosion = Some(texture_atlases.add(texture_atlas));

        material

    }
}
struct WinSize{
    #[allow(unused)]
    w: f32,
    h: f32,
}
struct ActiveEnemies(u32);

struct PlayerState{
    on: bool,
    last_shot: f64,
    invurnerable_timer: Timer,
    username: String,
    lifes: u32,
    score: u32,
}

impl Default for PlayerState{
    fn default() -> Self {
        Self{
            on: false,
            last_shot: 0.,
            invurnerable_timer: Timer::from_seconds(0.0, false),
            username: "<write down your name here>".to_string(),
            lifes: 3,
            score: 0,
        }
    }
}
impl PlayerState{
    fn shot_or_dead(&mut self, time: f64) -> bool{
        self.on = false;
        self.last_shot = time;
        self.invurnerable_timer = Timer::from_seconds(0.0, false);
        if self.lifes != 0{
            self.lifes -= 1;
        }
        else{
            return true
        }
        false
    }

    fn spawned(&mut self){
        self.on = true;
        self.last_shot = 0.;
        self.invurnerable_timer = Timer::from_seconds(1.5, false);
    }
}

struct GameOverToSpawn;
#[derive(Inspectable)]
struct GameOverText;

#[derive(Inspectable)]
struct ButtonSaveToDB;
struct ButtonSaveToDBLabel;

struct Laser;

struct Player;
struct PlayerReadyFire(bool);
struct FromPlayer;

#[derive(Inspectable, Default)]
struct Enemy;
struct FromEnemy;

struct Explosion;
struct ExplosionToSpawn(Vec3);

#[derive(Inspectable)]
struct PauseState(bool);
#[derive(Inspectable)]
struct GameState(String);

impl Default for PauseState{
    fn default() -> Self {
        PauseState(false)
    }
} 

#[derive(Inspectable)]
struct Speed{
    #[inspectable(min = 0.0, max = 1000.0)]
    v: f32,
}

impl Default for Speed{
    fn default() -> Self {
        Speed{v: 500.}
    }
}

#[derive(Inspectable)]
struct LaserSpeed{
    #[inspectable(min = 0.0, max = 1000.0)]
    v: f32,
}

impl Default for LaserSpeed{
    fn default() -> Self {
        LaserSpeed{v: 500.}
    }
}

#[derive(Inspectable, Default)]
struct PauseText;
struct CheatSheetTimer{
    timer: Timer,
}
impl Default for CheatSheetTimer{
    fn default() -> Self {
        CheatSheetTimer{
            timer: Timer::from_seconds(5.0, false),
        }
    }
}

fn main() {
    let mut app = App::build();


        app
        .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
        .insert_resource(GameState("active".to_string()))
        .insert_resource(WindowDescriptor{
            title: "Rust Invaders".to_string(),
            width: 598.,
            height: 676.,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0))
        .insert_resource(CheatSheetTimer::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .init_resource::<Materials>()
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(WindowPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(InspectorPlugin::<InspectorQuery<(Entity), With<Enemy>>>::new())
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<PauseText>>>::new())
        .add_plugin(InspectorPlugin::<InspectorQuery<Entity, (With<GameOverText>)>>::new())
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, (With<Player>)>>::new());

        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(InspectableRegistry::default);

        // registering custom component to be able to edit it in inspector
        registry.register::<Speed>();
        registry.register::<PauseState>();
        registry.register::<LaserSpeed>();
        registry.register::<GameOverText>();

        app.run();
}
