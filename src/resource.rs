use bevy::prelude::*;
use serde::Deserialize;

pub struct MyResourcePlugin;

impl Plugin for MyResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
            .insert_resource(FixedTime::new_from_secs(1.0))
            .init_resource::<Score>()
            .init_resource::<Config>();
    }
}

#[derive(Component)]
pub struct DisplayText(pub String);

#[derive(Component)]
pub struct QuestionActual(pub String);

#[derive(Component)]
pub struct QuestionTimer(pub Timer);

#[derive(Bundle)]
pub struct QuestionBundle {
    pub text: DisplayText,
    pub actual: QuestionActual,
    pub timer: QuestionTimer,
}

/**
 * 关卡配置
 */
#[derive(Debug, Deserialize, Resource, Default)]
pub struct Config {
    pub questions: Vec<Question>,
}

/**
 * 问题
 */
#[derive(Debug, Deserialize)]
pub struct Question {
    pub text: String,
    pub actual: String,
}

/**
 * 记分板
 */
#[derive(Debug, Default, Resource)]
pub struct Score {
    pub score: i32,
}

// 全局计时器
#[derive(Debug, Default, Resource)]
pub struct GlobalTimer(pub Timer);
