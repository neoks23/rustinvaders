use bevy::{core::FixedTimestep, prelude::*};

use crate::{Laser, Materials, Player, PlayerReadyFire, Speed, WinSize, SCALE, TIME_STEP, FromPlayer, PlayerState, PLAYER_RESPAWN_DELAY, PauseState, GameState, LaserSpeed, FIRING_SFX};
use bevy_inspector_egui::InspectableRegistry;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
    fn build(&self, app: &mut AppBuilder){

        app
            .insert_resource(PlayerState::default())
            .add_startup_stage(
            "game_setup_actors",
            SystemStage::single(player_spawn.system()),
        )
            .add_system(player_movement.system())
            .add_system(player_fire.system())
            .add_system(laser_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn.system())
            );
    }
}

fn player_spawn(mut commands: Commands,
                win_size: Res<WinSize>,
                materials: Res<Materials>,
                game_state: Res<GameState>,
                time: Res<Time>,
                mut player_state: ResMut<PlayerState>,)
{
    if game_state.0 != "gameover".to_string(){
        let now = time.seconds_since_startup();
        let last_shot = player_state.last_shot;

        //spawn a sprite

        if !player_state.on && (last_shot == 0. || now > last_shot + PLAYER_RESPAWN_DELAY){
            let bottom = -win_size.h / 2.;
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.player.clone(),
                    transform: Transform {
                        translation: Vec3::new(0., bottom + 75. / 4. + 5., 10.),
                        scale: Vec3::new(SCALE, SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Player)
                .insert(PlayerReadyFire(true))
                .insert(Speed::default())
                .insert(LaserSpeed::default())
                .insert(Timer::from_seconds(0.5, true))
                .insert(if game_state.0 == "active" {PauseState::default()} else {PauseState(true)});
            player_state.spawned();
        }
    }

}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(&PauseState, &Speed, &mut Transform, With<Player>)>
){

    if let Ok((pause, speed, mut transform, _)) = query.single_mut(){

        if !pause.0{
            let dir = if keyboard_input.pressed(KeyCode::A) && transform.translation.x - 75.0 > -win_size.w / 2.{
                -1.
            } else if keyboard_input.pressed(KeyCode::D) && transform.translation.x + 75.0 < win_size.w / 2.{
                1.
            } else{
                0.
            };
            transform.translation.x += dir * speed.v * TIME_STEP;
        }
    }
}

fn player_fire(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    kb: Res<Input<KeyCode>>,
    materials: Res<Materials>,
    mut query: Query<(&Transform,&PauseState,&LaserSpeed,&mut PlayerReadyFire, &mut Timer, With<Player>)>
){
    if let Ok((player_tf, pause_state,lspeed, mut ready_fire, mut timer, _)) = query.single_mut(){
        if ready_fire.0 && kb.pressed(KeyCode::Space) && !pause_state.0{
            let music = asset_server.load(FIRING_SFX);
            audio.play(music);
            let x = player_tf.translation.x;
            let y = player_tf.translation.y;

            let mut spawn_lasers = |x_offset: f32|{
                commands.spawn_bundle(SpriteBundle{
                    material: materials.player_laser.clone(),
                    transform: Transform{
                        translation: Vec3::new(x + x_offset, y + 15., 0.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(Speed{v: lspeed.v})
                    .insert(PauseState::default());;

            };
            let x_offset = 144.0 / 4.0 - 5.0;
            spawn_lasers(x_offset);
            spawn_lasers(-x_offset);

            ready_fire.0 = false;
        }

        if !ready_fire.0 {
            timer.tick(time.delta());
        }
        if kb.just_released(KeyCode::Space) || timer.finished(){
            ready_fire.0 = true;
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &PauseState, &Speed, &mut Transform, (With<Laser>, With<FromPlayer>))>
){
    for (laser_entity,pause, speed, mut laser_tf, _) in query.iter_mut(){

        if !pause.0{
            let translation = &mut laser_tf.translation;
            translation.y += speed.v * TIME_STEP;
            if translation.y > win_size.h{
                commands.entity(laser_entity).despawn();
            }
        }
    }
}