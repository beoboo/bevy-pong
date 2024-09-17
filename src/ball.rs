use bevy::color::palettes::basic::{BLUE, RED, WHITE};
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use rand::{Rng, thread_rng};
use crate::{AppState, Game, Paddle, PaddleHitSound, PaddleSide, ScoreSound, VIRTUAL_HEIGHT, VIRTUAL_WIDTH, WallHitSound};
use crate::paddle::PaddleSize;

const VICTORY_POINTS: u32 = 10;
const BALL_MIN_SPEED: f32 = 100.0;
const BALL_MAX_SPEED: f32 = 150.0;
const SPEED_DELTA: f32 = 1.2;

#[derive(Resource, Deref)]
pub struct BallTimer(Timer);

pub enum BallType {
    Regular,
    Grower,
    Shrinker,
}
#[derive(Component)]
pub struct Ball {
    pub(crate) speed: Vec2,
    ty: BallType,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            speed: Self::gen_speed(),
            ty: BallType::Regular,
        }
    }

    pub fn prepare_to_serve(&mut self, side: PaddleSide) {
        let mut rng = thread_rng();
        let dy = rng.gen_range(-50.0..=50.0);
        let dx = match side {
            PaddleSide::Left => rng.gen_range(BALL_MIN_SPEED..=BALL_MAX_SPEED),
            PaddleSide::Right => rng.gen_range(-BALL_MAX_SPEED..=-BALL_MIN_SPEED),
        };

        self.speed = Vec2::new(dx, dy);
    }

    fn gen_speed() -> Vec2 {
        let mut rng1 = thread_rng();
        let mut rng2 = thread_rng();

        let dx = if rng1.gen_range(1..=2) == 1 { 100.0 } else { -100.0 };
        let dy = if rng1.gen_range(1..=2) == 1 { rng2.gen_range(-100.0..-80.0) } else { rng2.gen_range(80.0..100.0) };

        Vec2::new(dx, dy)
    }

    pub fn color(&self) -> Color {
        match self.ty {
            BallType::Regular => WHITE.into(),
            BallType::Grower => BLUE.into(),
            BallType::Shrinker => RED.into(),
        }
    }
}

pub fn spawn_ball(commands: &mut Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(4.0, 4.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Ball::new());

    let mut rng = thread_rng();
    commands.insert_resource(BallTimer(Timer::from_seconds(rng.gen_range(1. ..10.), TimerMode::Once)))
}

pub fn update_ball(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Ball, &mut Sprite)>,
    mut timer: ResMut<BallTimer>,
    time: Res<Time>,
) {
    let (mut ball, mut sprite) = query.single_mut();

    timer.0.tick(time.delta());

    if timer.0.finished() {
        let mut rng = thread_rng();
        ball.ty = if rng.gen_bool(0.5) {
            BallType::Grower
        } else {
            BallType::Shrinker
        };

        let mut rng = thread_rng();
        commands.insert_resource(BallTimer(Timer::from_seconds(rng.gen_range(1. ..10.), TimerMode::Once)))
    }

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        ball.ty = BallType::Grower;
    }
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        ball.ty = BallType::Shrinker;
    }

    if ball.is_changed() {
        sprite.color = ball.color();
    }

}

pub fn move_ball(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    game: Res<Game>,
    mut query: Query<(&mut Transform, &mut Ball)>,
) {
    let (mut transform, mut ball) = query.single_mut();

    match app_state.get() {
        AppState::Play => {
            let dt = time.delta().as_secs_f32();
            transform.translation.x += ball.speed.x * dt;
            transform.translation.y += ball.speed.y * dt;
        },
        AppState::Serve => {
            ball.prepare_to_serve(game.serving_player);
        },
        AppState::Done => {
            transform.translation = Vec3::default();
        },
        _ => {}
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    mut paddles_q: Query<(&Transform, &mut Paddle), Without<Ball>>,
    mut ball_q: Query<(&mut Transform, &mut Ball)>,
    paddle_hit_sound: Res<PaddleHitSound>,
    wall_hit_sound: Res<WallHitSound>,
) {
    let (mut ball_transform, mut ball) = ball_q.single_mut();

    for (paddle_transform, mut paddle) in &mut paddles_q {
        if ball.speed.x > 0. && paddle.side == PaddleSide::Left {
            continue;
        }
        if ball.speed.x < 0. && paddle.side == PaddleSide::Right {
            continue;
        }

        let collision = Aabb2d::new(ball_transform.translation.xy(), ball_transform.scale.xy() / 2.)
            .intersects(&Aabb2d::new(paddle_transform.translation.xy(), paddle_transform.scale.xy() / 2.));

        if collision {
            let mut current_speed = ball.speed;
            let length = current_speed.length();

            current_speed.x *= -1.;

            let mut rng = thread_rng();
            let delta = rng.gen_range(10.0..=150.0);
            current_speed.y = if current_speed.y < 0.0 { -delta } else { delta };

            ball.speed = current_speed.normalize() * length * SPEED_DELTA;

            ball_transform.translation.x = match paddle.side {
                PaddleSide::Left => { paddle_transform.translation.x + paddle_transform.scale.x }
                PaddleSide::Right => { paddle_transform.translation.x - ball_transform.scale.x }
            };

            match ball.ty {
                BallType::Grower => paddle.size = PaddleSize::Large,
                BallType::Shrinker => paddle.size = PaddleSize::Small,
                BallType::Regular => paddle.size = PaddleSize::Regular,
            }

            ball.ty = BallType::Regular;

            commands.spawn(AudioBundle {
                source: paddle_hit_sound.clone(),
                settings: PlaybackSettings::DESPAWN,
                ..default()
            });
        }
    }

    let mid_height = VIRTUAL_HEIGHT as f32 / 2.0 - ball_transform.scale.y / 2.0;

    if ball_transform.translation.y < -mid_height {
        ball_transform.translation.y = -mid_height;
        ball.speed.y *= -1.;

        commands.spawn(AudioBundle {
            source: wall_hit_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
    }

    if ball_transform.translation.y > mid_height {
        ball_transform.translation.y = mid_height;
        ball.speed.y *= -1.;

        commands.spawn(AudioBundle {
            source: wall_hit_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
    }
}

pub fn score_point(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ball_q: Query<(&mut Transform, &mut Ball)>,
    score_sound: Res<ScoreSound>
) {
    let (mut ball_transform, mut ball) = ball_q.single_mut();

    if ball_transform.translation.x < -(VIRTUAL_WIDTH as f32 + ball_transform.scale.x) / 2.0 {
        game.serving_player = PaddleSide::Left;
        game.player2_score += 1;

        commands.spawn(AudioBundle {
            source: score_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });

        if game.player2_score == VICTORY_POINTS {
            game.winner = Some(PaddleSide::Right);
            next_state.set(AppState::Done);
        } else {
            ball.prepare_to_serve(game.serving_player);
            ball_transform.translation = Vec3::default();
            next_state.set(AppState::Serve);
        }
    }

    if ball_transform.translation.x > (VIRTUAL_WIDTH as f32 + ball_transform.scale.x) / 2.0 {
        game.serving_player = PaddleSide::Right;
        game.player1_score += 1;

        commands.spawn(AudioBundle {
            source: score_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });

        if game.player1_score == VICTORY_POINTS {
            game.winner = Some(PaddleSide::Left);
            next_state.set(AppState::Done);
        } else {
            ball.prepare_to_serve(game.serving_player);
            ball_transform.translation = Vec3::default();
            next_state.set(AppState::Serve);
        }
    }
}
