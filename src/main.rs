use bevy::{prelude::*, reflect::TypeUuid, utils::BoxedFuture};
use bevy_egui::{
    egui::{self, Align2, FontData, FontDefinitions},
    EguiContexts, EguiPlugin,
};
use rand::Rng;
use serde::Deserialize;

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};

/**
 * 关卡配置
 */
#[derive(Debug, Deserialize, Resource, Default, TypeUuid)]
#[uuid = "a5d6f4c3-6d3e-4f5f-9b9e-8a9d5f5c7d3e"]
struct Config {
    questions: Vec<Question>,
}

#[derive(Resource, Default)]
struct AssetsLoading(Vec<HandleUntyped>);

#[derive(Default)]
pub struct CustomAssetLoader;

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        let config = toml::from_str::<Config>(std::str::from_utf8(bytes).unwrap()).unwrap();

        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(config));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

/**
 * 问题
 */
#[derive(Debug, Deserialize)]
struct Question {
    text: String,
    actual: String,
}

/**
 * 记分板
 */
#[derive(Debug, Default, Resource)]
struct Store {
    score: i32,
}

/**
 * 记分板
 */
#[derive(Debug, Default, Resource)]
struct GlobalAssets {
    config: Handle<Config>,
}

/// 游戏状态
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum AppState {
    #[default]
    Loading,

    MainMenu,

    InGame,
}

/// 游戏状态
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    Idle,

    Playing,
    Paused,
    GameOver,
}

// 全局计时器
#[derive(Debug, Default, Resource)]
struct GlobalTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(GlobalTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(FixedTime::new_from_secs(1.0))
        .init_resource::<Store>()
        .init_resource::<GlobalAssets>()
        .init_resource::<AssetsLoading>()
        .add_state::<AppState>()
        .add_state::<GameState>()
        .add_asset::<Config>()
        .init_asset_loader::<CustomAssetLoader>()
        .add_startup_system(setup)
        .add_system(check_assets_ready.in_set(OnUpdate(AppState::Loading)))
        .add_system(render_main_menu.in_set(OnUpdate(AppState::MainMenu)))
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
        .add_system(render_paused_menu.in_set(OnUpdate(GameState::Paused)))
        .add_system(render_game_over.in_set(OnUpdate(GameState::GameOver)))
        .run();
}

/**
 * 每五秒随机生成一个问题
 */
fn spawn_question(
    mut commands: Commands,
    time: Res<Time>,
    global_assets: Res<GlobalAssets>,
    config_assets: ResMut<Assets<Config>>,
    mut global_timer: ResMut<GlobalTimer>,
) {
    if global_timer.0.tick(time.delta()).just_finished() {
        let config = config_assets.get(&global_assets.config).unwrap();

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
    mut score: ResMut<Store>,
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
    mut score: ResMut<Store>,
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

/// 渲染主菜单
fn render_main_menu(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    egui::Window::new("游戏菜单")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .hscroll(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        // 内容居中
        .default_width(120.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("主菜单");

            ui.separator();
            if ui.button("开始游戏").clicked() {
                app_state.set(AppState::InGame);
                game_state.set(GameState::Playing);
            }

            ui.separator();

            if ui.button("退出").clicked() {
                std::process::exit(0);
            }
        });
}

/// 渲染主菜单
fn render_paused_menu(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    egui::Window::new("暂停菜单")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .hscroll(false)
        .vscroll(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        // 内容居中
        .default_width(120.0)
        .show(contexts.ctx_mut(), |ui| {
            if ui.button("继续游戏").clicked() {
                game_state.set(GameState::Playing);
            }
            ui.separator();
            if ui.button("返回主菜单").clicked() {
                app_state.set(AppState::MainMenu);
            }
        });
}

fn render_game_over(mut contexts: EguiContexts, mut state: ResMut<NextState<GameState>>) {
    egui::Window::new("游戏结束").show(contexts.ctx_mut(), |ui| {
        ui.label("游戏结束");
        if ui.button("点击重新开始").clicked() {
            state.set(GameState::Playing);
        };
    });
}

/// 游戏结束系统
fn game_over(mut state: ResMut<NextState<GameState>>, score: ResMut<Store>) {
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

    commands.init_resource::<Store>()
}

/// 初始化资源
fn setup(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    mut global_assets: ResMut<GlobalAssets>,
    mut loading: ResMut<AssetsLoading>,
) {
    // 加载配置文件
    let config_handle: Handle<Config> = asset_server.load("config/config.toml");

    loading.0.push(config_handle.clone_untyped());
    global_assets.config = config_handle;

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "si_yuan".to_owned(),
        FontData::from_static(include_bytes!("../assets/font/SourceHanSansCN-Normal.otf")),
    ); // .ttf and .otf supported

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "si_yuan".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("si_yuan".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
}

fn check_assets_ready(
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    use bevy::asset::LoadState;

    match server.get_group_load_state(loading.0.iter().map(|h| h.id())) {
        LoadState::Failed => {
            // one of our assets had an error
        }
        LoadState::Loaded => {
            // all assets are now ready

            // this might be a good place to transition into your in-game state
            app_state.set(AppState::MainMenu);

            // remove the resource to drop the tracking handles
            // commands.remove_resource::<AssetsLoading>();
            // (note: if you don't have any other handles to the assets
            // elsewhere, they will get unloaded after this)
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}

/**
 * 渲染游戏界面
 */
fn print_info(
    mut contexts: EguiContexts,
    query: Query<(&DisplayText, &QuestionTimer)>,
    score: Res<Store>,
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

#[derive(Component)]
struct DisplayText(String);

#[derive(Component)]
struct QuestionActual(String);

#[derive(Component)]
struct QuestionTimer(Timer);

#[derive(Bundle)]
struct QuestionBundle {
    text: DisplayText,
    actual: QuestionActual,
    timer: QuestionTimer,
}
