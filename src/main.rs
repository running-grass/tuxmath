use bevy::prelude::*;
use bevy_egui::{
    egui::{self, FontData, FontDefinitions},
    EguiContexts, EguiPlugin,
};
use rand::Rng;
use serde::Deserialize;

/**
 * 关卡配置
 */
#[derive(Debug, Deserialize, Resource)]
struct Config {
    questions: Vec<Question>,
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

/// 游戏状态
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    MainMenu,

    Playing,
    Paused,
    GameOver,
}

// 全局计时器
#[derive(Debug, Default, Resource)]
struct GlobalTimer(Timer);

fn main() {
    let mut questions: Config = Config {
        questions: Vec::new(),
    };
    // 从config/config.toml文件中读取配置
    if let Ok(config) = std::fs::read_to_string("config/config.toml") {
        questions = toml::from_str(&config).unwrap();
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(questions)
        .insert_resource(GlobalTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(FixedTime::new_from_secs(1.0))
        .init_resource::<Store>()
        .add_state::<GameState>()
        .add_system(setup)
        // .add_startup_system(spawn_question)
        .add_system(render_main_menu.in_set(OnUpdate(GameState::MainMenu)))
        .add_system(render_game_over.in_set(OnUpdate(GameState::GameOver)))
        .add_system(reset_entity.in_schedule(OnExit(GameState::GameOver)))
        // 游戏中的系统
        .add_systems(
            (
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
    mut score: ResMut<Store>,
    mut query: Query<(Entity, &mut QuestionTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
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

/// 渲染主菜单
fn render_main_menu(mut contexts: EguiContexts, mut state: ResMut<NextState<GameState>>) {
    egui::Window::new("主菜单").show(contexts.ctx_mut(), |ui| {
        ui.label("欢迎来到TuxMath");
        if ui.button("点击开始游戏").clicked() {
            state.set(GameState::Playing);
        };
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
    mut score: ResMut<Store>,
    mut commands: Commands,
    query: Query<(Entity, &QuestionActual)>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }
    score.score = 0;
}

/// 初始化资源
fn setup(mut contexts: EguiContexts) {
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
