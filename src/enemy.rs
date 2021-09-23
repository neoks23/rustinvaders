use bevy::{core::FixedTimestep, prelude::*};
use crate::{WinSize, Materials, ActiveEnemies, Enemy, SCALE, Laser, FromEnemy, Speed, TIME_STEP, MAX_ENEMIES, MAX_FORMATION_MEMBERS, PauseState, GameState, LaserSpeed};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin};
use bevy_inspector_egui::widgets::{InspectorQuerySingle, InspectorQuery};

pub struct EnemyPlugin;

#[derive(Inspectable, Default, Clone)]
struct Formation{
    start: (f32, f32),
    radius: (f32, f32),
    offset: (f32, f32),
    angle: f32,
    group_id: u32,
}

#[derive(Inspectable, Default)]
struct FormationMaker{
    group_seq: u32,
    current_formation: Option<Formation>,
    current_formation_members: u32,
}

impl FormationMaker{
    fn make(&mut self, win_size: &WinSize) -> Formation {
        match(
            &self.current_formation,
            self.current_formation_members >= MAX_FORMATION_MEMBERS
        ){
            //if first formation or previous formation full
            (None, _) | (_, true) => {
                let mut rng = thread_rng();
                //compute start x/y
                let h_span = win_size.h / 2. - 100.;
                let w_span = win_size.w / 4.;
                let x = if rng.gen::<bool>() {
                    win_size.w
                } else {
                    -win_size.w
                };

                let y = rng.gen_range(-h_span..h_span) as f32;
                let start = (x, y);

                let offset = (rng.gen_range(-w_span..w_span), rng.gen_range(0.0..h_span));
                let radius = (rng.gen_range(80.0..150.), 100.);
                let angle: f32 = (y - offset.0).atan2(x - offset.1);

                self.group_seq += 1;
                let group_id = self.group_seq;
                let formation = Formation{
                    start,
                    offset,
                    radius,
                    angle,
                    group_id,
                };

                self.current_formation = Some(formation.clone());
                self.current_formation_members = 1;
                formation
            }
            //if still within the formation count
            (Some(tmpl), false) => {
                self.current_formation_members += 1;
                tmpl.clone()
            }
        }
    }
}

impl Plugin for EnemyPlugin{
    fn build(&self, app: &mut bevy::prelude::AppBuilder){
        app
            .insert_resource(FormationMaker::default())
            .add_system(enemy_laser_movement.system())
            .add_system(enemy_movement.system())
            .add_system(enemy_fire.system())
            .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(enemy_spawn.system()),
            );
    }
}

fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    mut formation_maker: ResMut<FormationMaker>,
    game_state: Res<GameState>,
    win_size: Res<WinSize>,
    materials: Res<Materials>
) {
    if active_enemies.0 < MAX_ENEMIES {

        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;

        commands
            .spawn_bundle(SpriteBundle{
                material: materials.enemy.clone(),
                transform: Transform{
                    translation: Vec3::new(x, y, 10.0),
                    scale: Vec3::new(SCALE, SCALE, 0.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(Speed::default())
            .insert(LaserSpeed::default())
            .insert(Timer::from_seconds(0.9, true))
            .insert(formation)
            .insert(if game_state.0 == "active" {PauseState::default()} else {PauseState(true)});

        active_enemies.0 += 1;
    }
}

fn enemy_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PauseState, &Speed, &mut Formation), With<Enemy>>
){
    //foreach enemy
    for (mut tf, pause, speed, mut formation) in query.iter_mut(){

        if !pause.0 {
            let max_distance = TIME_STEP * speed.v;
            let x_org = tf.translation.x;
            let y_org = tf.translation.y;

            //ellipse
            let (x_offset, y_offset) = formation.offset;
            let (x_radius, y_radius) = formation.radius;

            // Compute the next angle
            let dir = if formation.start.0 > 0. { 1. } else { -1. };
            let angle = formation.angle + dir * speed.v * TIME_STEP / (x_radius.min(y_radius) * PI / 2.);

            // Calculate destination
            let x_dst = x_radius * angle.cos() + x_offset;
            let y_dst = y_radius * angle.sin() + y_offset;

            //Calculate distance
            let dx = x_org - x_dst;
            let dy = y_org - y_dst;
            let distance = (dx * dx + dy * dy).sqrt();

            let distance_ratio = if distance == 0. {
                0.
            } else{
                max_distance / distance
            };

            //Calculate final x/y
            let x = x_org - dx * distance_ratio;
            let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };
            let y = y_org - dy * distance_ratio;
            let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

            // start rotating the formation angle only when sprite are on or close to destination
            if distance < max_distance * speed.v / 20. {
                formation.angle = angle;
            }

            tf.translation.x = x;
            tf.translation.y = y;
        }
    }
}

fn enemy_fire(
    mut commands: Commands,
    time: Res<Time>,
    materials: Res<Materials>,
    mut enemy_query: Query<(&Transform,&PauseState, &mut Timer, &LaserSpeed), With<Enemy>>
){
    for (&tf, pause,mut timer, lspeed) in enemy_query.iter_mut(){
        timer.tick(time.delta());
        if !pause.0 && timer.finished() {
            let x = tf.translation.x;
            let y = tf.translation.y;
            //spawn enemy laser sprite
            commands
                .spawn_bundle(SpriteBundle{
                    material: materials.enemy_laser.clone(),
                    transform: Transform{
                        translation: Vec3::new(x, y - 15., 0.),
                        scale: Vec3::new(SCALE, -SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Laser)
                .insert(FromEnemy)
                .insert(Speed{v: lspeed.v})
                .insert(PauseState::default());
        }
    }
}

fn enemy_laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut laser_query: Query<(Entity,&PauseState, &Speed, &mut Transform), (With<Laser>, With<FromEnemy>)>
){
    for (entity, pause, speed, mut tf) in laser_query.iter_mut() {
        if !pause.0{
            tf.translation.y -= speed.v * TIME_STEP;
            if tf.translation.y < -win_size.h / 2. - 50. {
                commands.entity(entity).despawn();
            }
        }
    }
}