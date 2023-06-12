use bevy::asset::*;

use bevy::prelude::*;
use bevy_egui::*;
use bevy_egui::egui::*;


use crate::Config;
// 引入state.rs 文件
use crate::state::{AppState};

pub struct MyAssetPlugin;

impl Plugin for MyAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<String>()
            .add_asset::<Vec<u8>>()
            .add_asset_loader(TomlAssetLoader)
            .add_asset_loader(RawLoader)
            .init_resource::<GlobalAssets>()
            .init_resource::<AssetsLoading>()

            .add_startup_system(load_asset)
            .add_system(check_assets_ready.in_set(OnUpdate(AppState::Loading)))
            ;
    }
}

#[derive(Debug, Default, Resource)]
struct GlobalAssets {
    config: Handle<String>,
    font: Handle<Vec<u8>>,
}

#[derive(Resource, Default)]
struct AssetsLoading(Vec<HandleUntyped>);

/**
 * 加载为String
 */
#[derive(Default)]
pub struct TomlAssetLoader;

impl AssetLoader for TomlAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        // let config = toml::from_str::<Config>(std::str::from_utf8(bytes).unwrap()).unwrap();

        let str = std::str::from_utf8(bytes).unwrap();

        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(str.to_string()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

// 自定义字体文件加载器
#[derive(Default)]
struct RawLoader;

impl AssetLoader for RawLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(bytes.to_vec()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["raw"]
    }
}

/// 初始化资源
fn load_asset(
    asset_server: Res<AssetServer>,
    mut global_assets: ResMut<GlobalAssets>,
    mut loading: ResMut<AssetsLoading>,
) {
    // 加载配置文件
    let config_handle: Handle<String> = asset_server.load("config/config.toml");
    let font_handle: Handle<Vec<u8>> = asset_server.load("font/SourceHanSansCN-Normal.otf.raw");
    loading.0.push(config_handle.clone_untyped());
    loading.0.push(font_handle.clone_untyped());
    global_assets.config = config_handle;
    global_assets.font = font_handle;
}



fn check_assets_ready(
  mut contexts: EguiContexts,
  server: Res<AssetServer>,
  loading: Res<AssetsLoading>,
  global_assets: Res<GlobalAssets>,

  string_assets: ResMut<Assets<String>>,
  bytes_assets: ResMut<Assets<Vec<u8>>>,
  mut app_state: ResMut<NextState<AppState>>,

  mut commands: Commands,
) {

  match server.get_group_load_state(loading.0.iter().map(|h| h.id())) {
      LoadState::Failed => {
          // one of our assets had an error
      }
      LoadState::Loaded => {
          // 设置关卡配置
          let config_str = string_assets.get(&global_assets.config).unwrap();

          let config = toml::from_str::<Config>(config_str).unwrap();

          commands.insert_resource(config);
          

          // 设置UI字体
          let font = bytes_assets.get(&global_assets.font).unwrap();
          let mut fonts = FontDefinitions::default();
          // font.arr
          fonts.font_data.insert(
              "si_yuan".to_owned(),
              FontData::from_owned(font.to_vec()),
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
          // this might be a good place to transition into your in-game state
          app_state.set(AppState::MainMenu);
          // remove the resource to drop the tracking handles
          commands.remove_resource::<AssetsLoading>();
          // (note: if you don't have any other handles to the assets
          // elsewhere, they will get unloaded after this)
      }
      _ => {
          // NotLoaded/Loading: not fully ready yet
      }
  }
}
