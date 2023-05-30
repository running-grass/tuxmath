use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

// use std::io::{stdout, Error, Write};

// use crossterm::{
//     cursor::{MoveTo, MoveToNextLine},
//     queue,
//     style::{Color, Print, ResetColor, SetBackgroundColor},
//     terminal::{Clear, ClearType},
// };

struct PressedEvent(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_event::<PressedEvent>()
        .add_startup_system(spawn_question)
        .add_system(unspawn_question)
        .insert_resource(FixedTime::new_from_secs(1.0))
        .add_system(print_info)
        .add_system(spawn_question.in_schedule(CoreSchedule::FixedUpdate))
        .run();
}

fn spawn_question(mut commands: Commands, time: Res<Time>) {
    if (time.elapsed_seconds() as u32) % 5 == 0 {
        commands.spawn(QuestionBundle {
            text: DisplayText("1+1=?".to_string()),
            actual: QuestionActual("2".to_string()),
            creation_time: CreationTime(time.elapsed_seconds() as u32),
        });
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
        println!("Got char: '{}'", ev.char);
        string.push(ev.char);
    }

    if keys.just_pressed(KeyCode::Return) {
        println!("Got char: return , string = '{}'", string.as_str());

        for (entity, actual) in query.iter() {
            println!("actual = '{}'", actual.0);
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
