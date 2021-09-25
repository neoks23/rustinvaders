// #![allow(unused)]

mod player;
mod enemy;
use bevy::prelude::*;
use crate::{enemy::EnemyPlugin, player::PlayerPlugin};
use bevy::sprite::collide_aabb::collide;
use std::collections::HashSet;
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin, InspectableRegistry};
use bevy_inspector_egui::widgets::{InspectorQuerySingle, InspectorQuery, ResourceInspector};
use bevy_inspector_egui::plugin::InspectorWindows;

const PLAYER_SPRITE: &str = "player_c_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_SPRITE: &str = "enemy_b_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const TIME_STEP: f32 = 1. / 60.;
const SCALE: f32 = 0.5;
const MAX_ENEMIES: u32 = 20;
const MAX_FORMATION_MEMBERS: u32 = 2;
const PLAYER_RESPAWN_DELAY: f64 = 2.;

pub struct Materials{
    player: Handle<ColorMaterial>,
    player_laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    enemy_laser: Handle<ColorMaterial>,
    explosion: Handle<TextureAtlas>,
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
}

impl Default for PlayerState{
    fn default() -> Self {
        Self{
            on: false,
            last_shot: 0.,
            invurnerable_timer: Timer::from_seconds(0.0, false),
        }
    }
}
impl PlayerState{
    fn shot(&mut self, time: f64){
        self.on = false;
        self.last_shot = time;
        self.invurnerable_timer = Timer::from_seconds(0.0, false);
    }

    fn spawned(&mut self){
        self.on = true;
        self.last_shot = 0.;
        self.invurnerable_timer = Timer::from_seconds(1.5, false);
    }
}

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
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(InspectorPlugin::<InspectorQuery<(Entity), With<Enemy>>>::new())
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<PauseText>>>::new())
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, (With<Player>)>>::new())
        .add_startup_system(setup.system())
        .add_startup_system(inspector_window_setup.system())
        .add_system(inspector_window.system())
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
        .add_system(pause_game.system());

        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(InspectableRegistry::default);

        // registering custom component to be able to edit it in inspector
        registry.register::<Speed>();
        registry.register::<PauseState>();
        registry.register::<LaserSpeed>();



        app.run();
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut materials: ResMut<Assets<ColorMaterial>>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
         mut windows: ResMut<Windows>,
){
    let mut window = windows.get_primary_mut().unwrap();
    //camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            visible: Visible{
                is_visible: true,
                is_transparent: false,
            },
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                visible: Visible{
                    is_visible: false,
                    is_transparent: false,
                },
                text: Text::with_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "pause",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 100.0,
                        color: Color::WHITE,
                    },
                    // Note: You can use `Default::default()` in place of the `TextAlignment`
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                ),
                style: Style {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(PauseText);
        });


    //create main resources
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0,64.0), 4, 4);

    commands.insert_resource(Materials{
        player: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        player_laser: materials.add(asset_server.load(PLAYER_LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        enemy_laser: materials.add(asset_server.load(ENEMY_LASER_SPRITE).into()),
        explosion: texture_atlases.add(texture_atlas),
    });
    commands.insert_resource(WinSize{w: window.height(), h: window.width()});

    //window

    //spawn a sprite
}
fn inspector_window_setup(
    mut inspector_windows: ResMut<InspectorWindows>
){
    let mut inspector_window_pause_data = inspector_windows.window_data_mut::<InspectorQuerySingle<Entity, With<PauseText>>>();
    inspector_window_pause_data.name = "Pause".to_string();
    inspector_window_pause_data.visible = false;
    let mut inspector_window_player_data = inspector_windows.window_data_mut::<InspectorQuerySingle<Entity, With<Player>>>();
    inspector_window_player_data.name = "Player".to_string();
    inspector_window_player_data.visible = false;
    let mut inspector_window_enemy_data = inspector_windows.window_data_mut::<InspectorQuery<(Entity), With<Enemy>>>();
    inspector_window_enemy_data.name = "Enemies".to_string();
    inspector_window_enemy_data.visible = false;
}
fn inspector_window(
    keyboard_input: Res<Input<KeyCode>>,
    mut cheat_sheet_timer: ResMut<CheatSheetTimer>,
    time: Res<Time>,
    mut inspector_windows: ResMut<InspectorWindows>
){

    if keyboard_input.pressed(KeyCode::I) && keyboard_input.pressed(KeyCode::R){
        cheat_sheet_timer.timer.tick(time.delta());
        if cheat_sheet_timer.timer.finished(){
            cheat_sheet_timer.timer.reset();
            let mut inspector_window_pause_data = inspector_windows.window_data_mut::<InspectorQuerySingle<Entity, With<PauseText>>>();
            inspector_window_pause_data.visible = !inspector_window_pause_data.visible;
            let mut inspector_window_player_data = inspector_windows.window_data_mut::<InspectorQuerySingle<Entity, With<Player>>>();
            inspector_window_player_data.visible = !inspector_window_player_data.visible;
            let mut inspector_window_enemy_data = inspector_windows.window_data_mut::<InspectorQuery<(Entity), With<Enemy>>>();
            inspector_window_enemy_data.visible = !inspector_window_enemy_data.visible;
        }
    }
}

fn player_laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform, &Sprite,(With<Laser>, With<FromPlayer>))>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>,
){
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();
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

                if enemies_blasted.get(&enemy_entity).is_none() {
                    // remove the enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;

                    //Spawn explosion
                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));

                    enemies_blasted.insert(enemy_entity);
                }

                // remove the laser
                commands.entity(laser_entity).despawn();

            }
        }
    }
}
fn enemy_laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut pause_query: Query<(&mut Visible, (With<PauseText>))>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &Sprite), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &Sprite), With<Player>>,
){


    for(mut visibility, _) in pause_query.iter_mut() {
        if player_state.on && !visibility.is_visible {
            player_state.invurnerable_timer.tick(time.delta());
            if player_state.invurnerable_timer.finished() {
                if let Ok((player_entity, player_tf, player_sprite)) = player_query.single() {
                    let player_size = player_sprite.size * Vec2::from(player_tf.scale.abs());

                    for (laser_entity, laser_tf, laser_sprite) in laser_query.iter() {
                        let laser_size = laser_sprite.size * Vec2::from(laser_tf.scale.abs());

                        let collision = collide(
                            laser_tf.translation,
                            laser_size,
                            player_tf.translation,
                            player_size,
                        );

                        if let Some(_) = collision {
                            commands.entity(player_entity).despawn();
                            player_state.shot(time.seconds_since_startup());

                            commands.entity(laser_entity).despawn();

                            commands
                                .spawn()
                                .insert(ExplosionToSpawn(player_tf.translation.clone()));
                        }
                    }
                }
            }
        }
    }
}
fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>,
){
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter(){
        commands
            .spawn_bundle(SpriteSheetBundle{
                texture_atlas: materials.explosion.clone(),
                transform: Transform{
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        With<Explosion>,
    )>,
){
    for(entity, mut timer, mut sprite, texture_atlas_handle, _) in query.iter_mut(){
        //subtract ticks from timer
        timer.tick(time.delta());

        //every 50 ms timer is finished
        if timer.finished(){
            //go to next frame
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;

            if sprite.index == texture_atlas.textures.len() as u32 {
                commands.entity(entity).despawn();
            }
        }
    }
}

//take this with a grain of salt
fn pause_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut pause_query: Query<(&mut Visible, (With<PauseText>))>,
    mut pause_state_query: QuerySet<(
    Query<&mut PauseState, With<Player>>,
    Query<&mut PauseState,(With<Laser>, With<FromPlayer>)>,
    Query<&mut PauseState, With<Enemy>>,
    Query<&mut PauseState, (With<Laser>, With<FromEnemy>)>,
    )>,
    mut game_state: ResMut<GameState>,
){
    for(mut visibility, _) in pause_query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            visibility.is_visible = !visibility.is_visible;
            if(visibility.is_visible){
                if let Ok((mut pause)) = pause_state_query.q0_mut().single_mut() {
                    pause.0 = true;
                }
                for mut pause in pause_state_query.q1_mut().iter_mut(){
                    pause.0 = true;
                }
                for mut pause in pause_state_query.q2_mut().iter_mut(){
                    pause.0 = true;
                }
                for mut pause in pause_state_query.q3_mut().iter_mut(){
                    pause.0 = true;
                }
                game_state.0 = "pause".to_string();
            }
            else{
                if let Ok((mut pause)) = pause_state_query.q0_mut().single_mut() {
                    pause.0 = false;
                }
                for mut pause in pause_state_query.q1_mut().iter_mut(){
                    pause.0 = false;
                }
                for mut pause in pause_state_query.q2_mut().iter_mut(){
                    pause.0 = false;
                }
                for mut pause in pause_state_query.q3_mut().iter_mut(){
                    pause.0 = false;
                }
                game_state.0 = "active".to_string();
            }

        }
    }
}
