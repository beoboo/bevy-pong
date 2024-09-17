use bevy::prelude::*;
use crate::PaddleSide;

#[derive(Resource)]
pub struct Game {
    pub serving_player: PaddleSide,
    pub winner: Option<PaddleSide>,
    pub player1_score: u32,
    pub player2_score: u32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            serving_player: PaddleSide::Left,
            winner: None,
            player1_score: 0,
            player2_score: 0,
        }
    }

    pub fn reset(&mut self) {
        self.player1_score = 0;
        self.player2_score = 0;
        self.serving_player = if self.winner.unwrap() == PaddleSide::Left { PaddleSide::Right } else { PaddleSide::Left };
        self.winner = None;
    }
}