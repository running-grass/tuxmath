use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource)]
struct Config {
    questions: Vec<Question>,
}

#[derive(Debug, Deserialize)]
struct Question {
    text: String,
    actual: String,
}

fn main() {
    let mut questions: Config = Config {
        questions: Vec::new(),
    };
    // 从config/config.toml文件中读取配置
    if let Ok(config) = std::fs::read_to_string("config/config.toml") {
        questions = toml::from_str(&config).unwrap();
        println!("{:?}", config);
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(questions)
        // .add_startup_system(spawn_question)
        .add_startup_system(spawn_question)
        .add_system(unspawn_question)
        .insert_resource(FixedTime::new_from_secs(1.0))
        .add_system(print_info)
        .add_system(spawn_question.in_schedule(CoreSchedule::FixedUpdate))
        .run();
}

fn spawn_question(mut commands: Commands, time: Res<Time>, config: Res<Config>) {
    if (time.elapsed_seconds() as u32) % 5 == 0 {
        let index: usize = rand::thread_rng().gen_range(0..config.questions.len()) as usize;

        // 从 question 随机取出一条
        let question = config.questions.get(index).unwrap();

        commands.spawn(QuestionBundle {
            text: DisplayText(question.text.to_string()),
            actual: QuestionActual(question.actual.to_string()),
            creation_time: CreationTime(time.elapsed_seconds() as u32),
        });

        debug!("spawn question: {:?}", question);

    }
}

fn unspawn_question(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut string: Local<String>,
    mut char_evr: EventReader<ReceivedCharacter>,

    query: Query<(Entity, &QuestionActual)>,
) {
    for ev in char_evr.iter() {
        string.push(ev.char);
    }

    if keys.just_pressed(KeyCode::Return) {
        for (entity, actual) in query.iter() {
            if string.trim().eq(&actual.0) {
                commands.entity(entity).despawn();
            }
        }
        string.clear();
    }
}

fn print_info(
    mut contexts: EguiContexts,
    time: Res<Time>,
    query: Query<(&DisplayText, &CreationTime)>,
) {
    contexts
        .ctx_mut()
        .set_fonts(egui::FontDefinitions::default());
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        for (text, creation) in query.iter() {
            ui.label(format!(
                "question: [{}], have {} s",
                text.0,
                time.elapsed_seconds() as u32 - creation.0
            ));
        }
    });
}

#[derive(Component)]
struct DisplayText(String);

#[derive(Component)]
struct QuestionActual(String);

#[derive(Component)]
struct CreationTime(u32);

#[derive(Bundle)]
struct QuestionBundle {
    text: DisplayText,
    actual: QuestionActual,
    creation_time: CreationTime,
}
