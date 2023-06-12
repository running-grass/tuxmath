use bevy::prelude::*;

/// 应用状态
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Loading,

    MainMenu,

    InGame,
}

/// 游戏状态
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    #[default]
    Idle,

    Playing,
    Paused,
    GameOver,
}