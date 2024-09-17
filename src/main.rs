use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::ball::{Ball, handle_collisions, move_ball, score_point, spawn_ball, update_ball};
use crate::game::Game;
use crate::paddle::{update_paddle, move_paddles, Paddle, PaddleSide, spawn_paddle};

// use crate::pixel_camera::PixelCameraBundle;

// mod pixel_camera;
mod paddle;
mod ball;
mod game;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

const VIRTUAL_WIDTH: i32 = 432;
const VIRTUAL_HEIGHT: i32 = 243;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Start,
    Serve,
    Play,
    Done,
}

#[derive(Component)]
struct Title1Label;

#[derive(Component)]
struct Title2Label;

#[derive(Component)]
struct Score1Label;

#[derive(Component)]
struct Score2Label;

#[derive(Component)]
struct SpeedLabel;

#[derive(Resource, Deref)]
pub struct PaddleHitSound(Handle<AudioSource>);
#[derive(Resource, Deref)]
pub struct ScoreSound(Handle<AudioSource>);
#[derive(Resource, Deref)]
pub struct WallHitSound(Handle<AudioSource>);


fn main() {
    App::new()
        // Clears the screen with a specific color, similar to the original version of Pong
        .insert_resource(ClearColor(Color::srgb(40.0 / 255.0, 45.0 / 255.0, 52.0 / 255.0)))
        .insert_resource(Game::new())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pong".into(),
                // width: WINDOW_WIDTH,
                // height: WINDOW_HEIGHT,
                resizable: true,
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                ..default()
            }),
            ..default()
        }).build())
        // .add_plugins(WorldInspectorPlugin::default())
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            move_paddles,
            move_ball,
            show_speed,
            show_titles,
            show_score,
        ))
        .add_systems(
            Update, (
                update_paddle,
                update_ball,
                handle_collisions,
                score_point,
            ).run_if(in_state(AppState::Play)),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 0.35,
            ..default()
        },
        ..default()
    });

    let font = asset_server.load("fonts/retro.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 8.0,
        color: Color::WHITE,
    };
    let speed_text_style = TextStyle {
        font: font.clone(),
        font_size: 8.0,
        color: GREEN.into(),
    };

    let score_text_style = TextStyle {
        font: font.clone(),
        font_size: 32.0,
        color: Color::WHITE,
    };

    // Sounds
    commands.insert_resource(PaddleHitSound(asset_server.load("sounds/paddle_hit.ogg")));
    commands.insert_resource(WallHitSound(asset_server.load("sounds/wall_hit.ogg")));
    commands.insert_resource(ScoreSound(asset_server.load("sounds/score.ogg")));

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("Hello", text_style.clone()).with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, VIRTUAL_HEIGHT as f32 / 2.0 - 10.0, 0.0),
            ..default()
        })
        .insert(Title1Label)
    ;

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("World", text_style.clone()).with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, VIRTUAL_HEIGHT as f32 / 2.0 - 20.0, 0.0),
            ..default()
        })
        .insert(Title2Label)
    ;

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("Speed: 0.0", speed_text_style.clone()).with_justify(JustifyText::Left),
            transform: Transform::from_xyz(-VIRTUAL_WIDTH as f32 / 2.0 + 10.0, VIRTUAL_HEIGHT as f32 / 2.0 - 10.0, 0.0),
            ..default()
        })
        .insert(SpeedLabel)
    ;

    // Left score
    commands
        .spawn(Text2dBundle {
            text: Text::from_section("0", score_text_style.clone()).with_justify(JustifyText::Center),
            transform: Transform::from_xyz(-40.0, 25.0, 0.0),
            ..default()
        })
        .insert(Score1Label)
    ;

    // Right score
    commands
        .spawn(Text2dBundle {
            text: Text::from_section("0", score_text_style.clone()).with_justify(JustifyText::Center),
            transform: Transform::from_xyz(40.0, 25.0, 0.0),
            ..default()
        })
        .insert(Score2Label)
    ;

    // Left paddle
    spawn_paddle(&mut commands, PaddleSide::Left, -VIRTUAL_WIDTH as f32 / 2.0 + 10.0, VIRTUAL_HEIGHT as f32 / 2.0 - 40.0);

    // Right paddle
    spawn_paddle(&mut commands, PaddleSide::Right, VIRTUAL_WIDTH as f32 / 2.0 - 10.0, -VIRTUAL_HEIGHT as f32 / 2.0 + 40.0);

    // Ball
    spawn_ball(&mut commands);
}

fn show_speed(
    mut ball_q: Query<&Ball>,
    mut text_q: Query<&mut Text, With<SpeedLabel>>,
) {
    let ball = ball_q.single();
    let mut text = text_q.single_mut();

    text.sections[0].value = format!("Speed: {:.1}", ball.speed.length());
}

fn show_titles(
    app_state: Res<State<AppState>>,
    game: Res<Game>,
    mut query1: Query<&mut Text, (With<Title1Label>, Without<Title2Label>)>,
    mut query2: Query<&mut Text, With<Title2Label>>,
) {
    let mut text1 = query1.single_mut();
    let mut text2 = query2.single_mut();

    match app_state.get() {
        AppState::Start => {
            text1.sections[0].value = "Welcome to Pong!".into();
            text2.sections[0].value = "Press SPACE to begin!".into();
        }
        AppState::Serve => {
            text1.sections[0].value = format!("Player {}'s serve! ", game.serving_player);
            text2.sections[0].value = "Press SPACE to serve!".into();
        }
        AppState::Play => {
            text1.sections[0].value = "".to_string();
            text2.sections[0].value = "".to_string();
        }
        AppState::Done => {
            if let Some(winner) = game.winner {
                text1.sections[0].value = format!("Player {} wins!", winner);
                text2.sections[0].value = "Press SPACE to restart!".to_string();
            }
        }
    }
}

fn show_score(
    game: Res<Game>,
    mut query1: Query<&mut Text, (With<Score1Label>, Without<Score2Label>)>,
    mut query2: Query<&mut Text, With<Score2Label>>,
) {
    let mut text1 = query1.single_mut();
    let mut text2 = query2.single_mut();
    text1.sections[0].value = game.player1_score.to_string();
    text2.sections[0].value = game.player2_score.to_string();
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        match app_state.get() {
            AppState::Start => next_state.set(AppState::Serve),
            AppState::Serve => next_state.set(AppState::Play),
            AppState::Done => {
                next_state.set(AppState::Serve);
                game.reset();
            }
            _ => {}
        }
    }
}
