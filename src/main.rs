pub mod asset;
pub mod menu;
pub mod resource;
pub mod state;

use asset::*;
use bevy::{prelude::*, log::LogPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use resource::*;

use menu::*;
use rand::Rng;
use state::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,tuxmath=trace".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(MyAssetPlugin)
        .add_plugin(MyMenuPlugin)
        .add_plugin(MyStatePlugin)
        .add_plugin(MyResourcePlugin)
        .add_system(enter_game_system.in_schedule(OnEnter(AppState::InGame)))

        .add_system(reset_entity.in_schedule(OnExit(AppState::InGame)))
        // 游戏中的系统
        .add_systems(
            (
                paused_system,
                spawn_question,
                clean_question,
                unspawn_question,
                print_info,
                game_over,
            )
                .in_set(OnUpdate(GameState::Playing)),
        )
        .run();
}


fn enter_game_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    debug!("enter game system");
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/night_sky.jpg"),
        ..default()
    });

}

/**
 * 每五秒随机生成一个问题
 */
fn spawn_question(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<Config>,
    mut global_timer: ResMut<GlobalTimer>,
) {
    if global_timer.0.tick(time.delta()).just_finished() {
        let index: usize = rand::thread_rng().gen_range(0..config.questions.len()) as usize;
        // 从 question 随机取出一条
        let question = config.questions.get(index).unwrap();

        commands.spawn(QuestionBundle {
            text: DisplayText(question.text.to_string()),
            actual: QuestionActual(question.actual.to_string()),
            timer: QuestionTimer(Timer::from_seconds(20.0, TimerMode::Once)),
        });

        debug!("spawn question: {:?}", question);
    }
}

/**
 * 清理过时的问题
 */
fn clean_question(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut query: Query<(Entity, &mut QuestionTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
            score.score -= 1;
        }
    }
}

/**
 * 监听按键
 */
fn unspawn_question(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut string: Local<String>,
    mut char_evr: EventReader<ReceivedCharacter>,
    query: Query<(Entity, &QuestionActual)>,
    mut score: ResMut<Score>,
) {
    for ev in char_evr.iter() {
        string.push(ev.char);
    }

    if keys.just_pressed(KeyCode::Return) {
        for (entity, actual) in query.iter() {
            if string.trim().eq(&actual.0) {
                commands.entity(entity).despawn();

                score.score += 1;
            }
        }
        string.clear();
    }
}

/// .
fn paused_system(keys: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
    }
}

/// 游戏结束系统
fn game_over(mut state: ResMut<NextState<GameState>>, score: ResMut<Score>) {
    if score.score >= 0 && score.score <= 2 {
        return;
    }

    state.set(GameState::GameOver);
}

// 重置资源
fn reset_entity(
    mut commands: Commands,
    query: Query<(Entity, &QuestionActual)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_state.set(GameState::Idle);

    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }

    commands.init_resource::<Score>()
}

/**
 * 渲染游戏界面
 */
fn print_info(
    mut contexts: EguiContexts,
    query: Query<(&DisplayText, &QuestionTimer)>,
    score: Res<Score>,
) {
    egui::Window::new("问题列表").show(contexts.ctx_mut(), |ui| {
        for (text, timer) in query.iter() {
            ui.label(format!(
                "question: [{}], have {} s",
                text.0,
                timer.0.remaining_secs() as i32
            ));
        }
    });

    egui::Window::new("记分板").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("score: [{}]", score.score));
    });
}
