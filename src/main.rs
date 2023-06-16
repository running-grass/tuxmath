pub mod asset;
pub mod menu;
pub mod resource;
pub mod state;

use asset::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy::{log::LogPlugin, prelude::*};

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use resource::*;

use menu::*;
use rand::Rng;
use state::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,tuxmath=trace".into(),
                    level: bevy::log::Level::DEBUG,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tuxmath".to_string(),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(EguiPlugin)
        .add_plugin(MyAssetPlugin)
        .add_plugin(MyMenuPlugin)
        .add_plugin(MyStatePlugin)
        .add_plugin(MyResourcePlugin)
        .add_system(enter_game_system.in_schedule(OnEnter(AppState::InGame)))
        .add_system(reset_entity.in_schedule(OnExit(AppState::InGame)))
        .add_system(reset_entity.in_schedule(OnExit(GameState::GameOver)))
        // 游戏中的系统
        .add_systems(
            (
                paused_system,
                spawn_question,
                animate_sprite,
                clean_question,
                unspawn_question,
                print_info,
                game_over,
            )
                .in_set(OnUpdate(GameState::Playing)),
        )
        .run();
}

fn enter_game_system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    debug!("enter game system");
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/night_sky.jpg"),
        // transform: Transform::from_translation(Vec3::NEG_Z),
        ..Default::default()
    });
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &mut Transform,
        &QuestionTimer,
    )>,
) {
    for (mut sprite, mut timer, mut trans, q_timer) in &mut query {
        trans.translation.y = q_timer.0.remaining_secs() * 10.0;

        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == 7 {
                0
            } else {
                sprite.index + 1
            };
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/**
 * 每五秒随机生成一个问题
 */
fn spawn_question(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<Config>,
    mut global_timer: ResMut<GlobalTimer>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut texture_atlas_handle: Local<Option<Handle<TextureAtlas>>>,
) {
    let texture_atlas_handle = texture_atlas_handle
        .get_or_insert_with(|| {
            let texture_handle = asset_server.load("image/meteor.png");
            let texture_atlas =
                TextureAtlas::from_grid(texture_handle, Vec2::new(136.0, 225.0), 1, 8, None, None);

            texture_atlases.add(texture_atlas)
        })
        .to_owned();

    if global_timer.0.tick(time.delta()).just_finished() {
        let index: usize = rand::thread_rng().gen_range(0..config.questions.len()) as usize;
        // 从 question 随机取出一条
        let question = config.questions.get(index).unwrap();

        let half_w = WINDOW_WIDTH / 2.0;

        // 初始化文字信息
        let font = asset_server.load("font/SourceHanSansCN-Normal.otf");
        let text_style = TextStyle {
            font,
            font_size: 40.0,
            color: Color::BLUE,
            // ..Default::default()
        };

        commands
            .spawn(QuestionBundle {
                text: DisplayText(question.text.to_string()),
                actual: QuestionActual(question.actual.to_string()),
                timer: QuestionTimer(Timer::from_seconds(20.0, TimerMode::Once)),
            })
            .insert(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_translation(Vec3::new(
                    rand::thread_rng().gen_range(-half_w..half_w),
                    WINDOW_HEIGHT / 2.0,
                    10.0,
                )),
                ..default()
            })
            .insert(AnimationTimer(Timer::from_seconds(
                0.1,
                TimerMode::Repeating,
            )))
            .with_children(|question_box| {
                question_box.spawn(Text2dBundle {
                    text: Text::from_section(question.text.to_string(), text_style.clone())
                        .with_alignment(TextAlignment::Center),
                    transform: Transform::from_translation(Vec3::new(0.0, -50.0, 20.0)),
                    ..default()
                });
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
            commands.entity(entity).despawn_recursive();
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
                commands.entity(entity).despawn_recursive();

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
    if score.score >= -3 && score.score < 5 {
        return;
    }

    debug!("game over");

    state.set(GameState::GameOver);
}

// 重置资源
fn reset_entity(mut commands: Commands, query: Query<(Entity, &QuestionActual)>) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    debug!("reset entity");
    commands.insert_resource::<Score>(Score { score: 0 })
}

/**
 * 渲染游戏界面
 */
fn print_info(mut contexts: EguiContexts, score: Res<Score>) {
    egui::Window::new("记分板").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("score: [{}]", score.score));
    });
}
