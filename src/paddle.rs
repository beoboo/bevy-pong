use bevy::prelude::*;
use crate::VIRTUAL_HEIGHT;
use std::fmt::{Display, Formatter};
use bevy::color::palettes::css::*;

const PADDLE_SPEED: f32 = 200.0;
const PADDLE_HEIGHT: f32 = 30.0;
const PADDLE_WIDTH: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaddleSide {
    Left,
    Right,
}

impl Display for PaddleSide {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PaddleSide::Left => write!(f, "1"),
            PaddleSide::Right => write!(f, "2"),
        }
    }
}

pub enum PaddleSize {
    Small,
    Regular,
    Large,
}

#[derive(Component)]
pub struct Paddle {
    pub score: i32,
    pub side: PaddleSide,
    pub size: PaddleSize,
}

impl Paddle {
    pub fn new(side: PaddleSide) -> Self {
        Self {
            score: 0,
            side,
            size: PaddleSize::Regular,
        }
    }

    pub fn scale(&self) -> Vec3 {
        match self.size {
            PaddleSize::Small => Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT * 0.6, 0.),
            PaddleSize::Regular => Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 0.),
            PaddleSize::Large => Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT * 1.5, 0.),
        }
    }

    pub fn color(&self) -> Color {
        match self.size {
            PaddleSize::Small => RED.into(),
            PaddleSize::Regular => WHITE.into(),
            PaddleSize::Large => BLUE.into(),
        }
    }
}

pub fn spawn_paddle(commands: &mut Commands, side: PaddleSide, width: f32, height: f32) {
    let paddle = Paddle::new(side);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(width, height, 0.0),
                scale: paddle.scale(),
                ..default()
            },
            ..default()
        })
        .insert(paddle);
}

pub fn update_paddle(mut query: Query<(Ref<Paddle>, &mut Transform, &mut Sprite)>) {
    for (paddle, mut transform, mut sprite) in &mut query {
        if paddle.is_changed() {
            transform.scale = paddle.scale();
            sprite.color = paddle.color();
        }
    }
}

pub fn move_paddles(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Paddle)>,
) {
    let allowed_y = VIRTUAL_HEIGHT as f32 / 2.0 - PADDLE_HEIGHT / 2.0;
    let dt = time.delta().as_secs_f32();

    for (mut transform, mut paddle) in query.iter_mut() {
        match paddle.side {
            PaddleSide::Left => {
                if keyboard_input.pressed(KeyCode::KeyW) {
                    transform.translation.y += PADDLE_SPEED * dt;
                }
                if keyboard_input.pressed(KeyCode::KeyS) {
                    transform.translation.y -= PADDLE_SPEED * dt;
                }
                transform.translation.y = transform.translation.y.clamp(-allowed_y, allowed_y);
            }
            PaddleSide::Right => {
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    transform.translation.y += PADDLE_SPEED * dt;
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    transform.translation.y -= PADDLE_SPEED * dt;
                }
                transform.translation.y = transform.translation.y.clamp(-allowed_y, allowed_y);
            }
        }
    }
}
