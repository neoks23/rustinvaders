use bevy::prelude::*;
use bevy_egui::{EguiContext, egui};
use crate::{PlayerState, GameState, Materials, ButtonSaveToDB, ButtonSaveToDBLabel};
use futures::executor::block_on;
use sqlx::mysql::MySqlPoolOptions;

pub struct UiPlugin;

impl Plugin for UiPlugin{
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(ui_text_box.system())
            .add_system(button_system.system());
    }
}

fn ui_text_box(
    mut egui_ctx: ResMut<EguiContext>,
    mut player_state: ResMut<PlayerState>,
    assets: Res<AssetServer>,
    game_state: Res<GameState>,
) {
    if game_state.0 == "gameover" {
        egui::Area::new("my_area")
            .fixed_pos(egui::pos2(370.0, 450.0))
            .show(egui_ctx.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut player_state.username);
                });
            });
    }
}

fn button_system(
    materials: Res<Materials>,
    game_state: Res<GameState>,
    mut player_state: ResMut<PlayerState>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<ButtonSaveToDB>),
    >,
    mut text_query: Query<&mut Text>,
    mut label_query: Query<&mut Visible, With<ButtonSaveToDBLabel>>,
){
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        for (mut visible) in label_query.iter_mut(){
            let mut text = text_query.get_mut(children[0]).unwrap();
            if game_state.0 == "gameover"{
                visible.is_visible = true;
            }
            if visible.is_visible{
                match *interaction{
                    Interaction::Clicked => {
                        text.sections[0].value = "Name:\nScore: ".to_owned() + player_state.score.to_string().as_str() + &"\nSave to DB".to_string();
                        *material = materials.pressed.clone();
                        text.sections[0].style.color = Color::rgb(0.1,0.9,0.1);

                        let future = save_to_db(&player_state.username, player_state.score);

                        let result = block_on(future);

                        match result{
                            Ok(_) => player_state.username = "score saved!".to_string(),
                            Err(_) => player_state.username = "something went wrong".to_string()
                        }
                    }
                    Interaction::Hovered => {
                        text.sections[0].value = "Name:\nScore: ".to_owned()  + player_state.score.to_string().as_str() + &"\nSave to DB".to_string();
                        *material = materials.hovered.clone();
                        text.sections[0].style.color = Color::rgb(0.8,0.8,0.8);
                    }
                    Interaction::None => {
                        text.sections[0].value = "Name:\nScore: ".to_owned()  + player_state.score.to_string().as_str() + &"\nSave to DB".to_string();
                        *material = materials.normal.clone();
                        text.sections[0].style.color = Color::rgb(0.9,0.9,0.9);
                    }
                }
            }
        }
    }
}

async fn save_to_db(username: &str, score: u32) -> Result<(), sqlx::Error>{
    let pool = MySqlPoolOptions::new().max_connections(5).connect("mysql://localhost/gildaga").await?;

    sqlx::query("INSERT INTO score (Username, Score) VALUES ( ?, ? )").bind(username).bind(score).execute(&pool).await?;
    Ok(())
}