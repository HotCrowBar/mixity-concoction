use bevy::window::WindowResolution;

use crate::prelude::*;

mod assets;
mod game;
mod main_menu;
mod helper;

#[cfg(debug_assertions)]
mod debug;

fn main() {
	let mut app = App::default();

	app.add_state::<GameState>()
		.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(
					bevy_embedded_assets::EmbeddedAssetPlugin,
				)
				.set(bevy::prelude::WindowPlugin {
					primary_window: Some(Window {
						title: "Project Concoction".to_string(),
						present_mode: bevy::window::PresentMode::Immediate,
						resolution: WindowResolution::new(600.0, 350.0),
						..default()
					}),
					..default()
				}),
		)
		.add_plugin(game::GamePlugin)
		.add_plugin(assets::AssetLoadingPlugin);

	#[cfg(debug_assertions)]
	app.add_plugin(debug::DebugPlugin);

	app.run();
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum GameState {
	#[default]
	LoadingAssets,
	MainMenu,
	InGame,
	GeneratingWorld,
}

pub mod prelude {
	pub use bevy::prelude::*;
	pub use bevy_rapier3d::prelude::*;
	pub use leafwing_input_manager::prelude::*;
	pub use rand::prelude::*;
	pub use std::f32::consts::PI;
	pub use smallvec::SmallVec;
	
	pub use crate::helper::*;
	pub use crate::assets::GameAssets;
	pub use crate::game::input::*;
	pub use crate::game::materials::*;
	pub use crate::GameState;

	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::prelude::*;
	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::quick::WorldInspectorPlugin;
	#[cfg(debug_assertions)]
	pub use bevy_prototype_debug_lines::*;
}
