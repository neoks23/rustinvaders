use bevy::prelude::*;
use crate::{GameState, PlayerState, Laser, FromPlayer, Enemy, ActiveEnemies, KILL_SFX, ExplosionToSpawn, PauseText, FromEnemy, Player, GameOverToSpawn, GAMEOVER_SFX, DEAD_SFX, Materials, Explosion, GameOverText, PauseState};
use std::collections::HashSet;
use bevy::sprite::collide_aabb::collide;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(player_laser_hit_enemy.system())
            .add_system(enemy_laser_hit_player.system())
            .add_system(explosion_to_spawn.system())
            .add_system(animate_explosion.system())
            .add_system(pause_game.system())
            .add_system(gameover_to_spawn.system());
    }
}

fn player_laser_hit_enemy(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut player_state: ResMut<PlayerState>,
    mut laser_query: Query<(Entity, &Transform, &Sprite,(With<Laser>, With<FromPlayer>))>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>,
){
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();
    for (enemy_entity, enemy_tf, enemy_sprite, _) in enemy_query.iter_mut(){
        if game_state.0 == "gameover"{
            commands.entity(enemy_entity).despawn();
            active_enemies.0 = 0;
        }
    }

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
                    player_state.score += 1;

                    let music = asset_server.load(KILL_SFX);
                    audio.play(music);
                    //Audio::play(music, ());

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
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
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
                            if player_state.shot_or_dead(time.seconds_since_startup()) {
                                commands
                                    .spawn()
                                    .insert(GameOverToSpawn);

                                game_state.0 = "gameover".to_string();
                            }
                            else{
                                let music = asset_server.load(DEAD_SFX);
                                audio.play(music);
                            }
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
                texture_atlas: materials.explosion.clone().unwrap(),
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

fn gameover_to_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &GameOverToSpawn)>,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
){

    for (gameover_spawn_entity, gameover_to_spawn) in query.iter() {
        commands
            .spawn_bundle(NodeBundle {
                visible: Visible {
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
                    visible: Visible {
                        is_visible: true,
                        is_transparent: false,
                    },
                    text: Text::with_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "Game Over",
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
                    .insert(GameOverText);
            });
        commands.entity(gameover_spawn_entity).despawn();
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
        if game_state.0 != "gameover"{
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
}