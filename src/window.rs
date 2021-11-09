use bevy::prelude::*;
use crate::{Materials, WinSize, PauseText, ButtonSaveToDBLabel, ButtonSaveToDB, Player, Enemy, GameOverText, CheatSheetTimer, GameState, PlayerState};
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::widgets::{InspectorQuerySingle, InspectorQuery};
use sqlx::mysql::MySqlPoolOptions;
use futures::executor::block_on;
use bevy_egui::{egui, EguiContext};

pub struct WindowPlugin;

impl Plugin for WindowPlugin{
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_startup_system(inspector_window_setup.system())
            .add_system(inspector_window.system());
    }
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut cmaterials: ResMut<Assets<ColorMaterial>>,
         materials: Res<Materials>,
         mut windows: ResMut<Windows>,
){
    let mut window = windows.get_primary_mut().unwrap();

    commands.insert_resource(WinSize{w: window.height(), h: window.width()});

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
            material: cmaterials.add(Color::NONE.into()),
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

    commands
        .spawn_bundle(ButtonBundle {
            visible: Visible{
                is_visible: true,
                is_transparent: false,
            },
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(150.0)),
                // center button
                margin: Rect{
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Px(100.0),
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.pressed.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                visible: Visible{
                    is_visible: false,
                    is_transparent: false,
                },
                text: Text::with_section(
                    "Button",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
                .insert(ButtonSaveToDBLabel);
        })
        .insert(ButtonSaveToDB);
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
    let mut inspector_window_gameover_data = inspector_windows.window_data_mut::<InspectorQuery<(Entity), With<GameOverText>>>();
    inspector_window_gameover_data.name = "Game Over".to_string();
    inspector_window_gameover_data.visible = false;
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
            let mut inspector_window_gameover_data = inspector_windows.window_data_mut::<InspectorQuery<(Entity), With<GameOverText>>>();
            inspector_window_gameover_data.visible = !inspector_window_gameover_data.visible;
        }
    }
}