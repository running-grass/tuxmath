use bevy::prelude::Image as BevyImage;
use bevy::prelude::*;

use bevy_egui::egui::{self, Align2, ColorImage, Image, TextureHandle};
use bevy_egui::*;

// 引入state.rs 文件
use crate::state::*;
pub struct MyMenuPlugin;

impl Plugin for MyMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuHandles>()
            .add_system(render_main_menu.in_set(OnUpdate(AppState::MainMenu)))
            .add_system(render_paused_menu.in_set(OnUpdate(GameState::Paused)))
            .add_system(render_game_over.in_set(OnUpdate(GameState::GameOver)));
    }
}

#[derive(Debug, Default, Resource)]
pub struct MainMenuHandles {
    pub backend_image: Handle<BevyImage>,
}


/// 渲染主菜单
fn render_main_menu(
    mut egui_context: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    image_assets: ResMut<Assets<BevyImage>>,
    main_menu_handles: Res<MainMenuHandles>,
    mut image_texture_handle: Local<Option<TextureHandle>>,
) {
    let texture = image_texture_handle.get_or_insert_with(|| {
        let img = image_assets.get(&main_menu_handles.backend_image).unwrap();

        let img_data = ColorImage::from_rgba_unmultiplied(
            [
                img.texture_descriptor.size.width as usize,
                img.texture_descriptor.size.height as usize,
            ],
            &img.data,
        );

        let texture_handle =
            egui_context
                .ctx_mut()
                .load_texture("backend_image", img_data, Default::default());

        debug!("加载主菜单背景图到本地资源");

        texture_handle
    });

    // let central_panel = CentralPanel::default();
    egui::Window::new("游戏背景")
        .title_bar(false)
        .auto_sized()
        .interactable(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("主菜单");

            let img_size = texture.size_vec2();

            ui.add(Image::new(texture, img_size));
        });

    egui::Window::new("游戏主菜单")
        .title_bar(false)
        // .(1)
        .auto_sized()
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        // 内容居中
        .default_width(120.0)
        .show(egui_context.ctx_mut(), |ui| {
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

fn render_game_over( mut contexts: EguiContexts, mut state: ResMut<NextState<GameState>>) {
    egui::Window::new("游戏结束").show(contexts.ctx_mut(), |ui| {
        ui.label("游戏结束");

        if ui.button("点击重新开始").clicked() {
            state.set(GameState::Playing);
        };

    });
}
