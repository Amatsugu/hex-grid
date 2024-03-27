use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
mod hex_grid;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_perf_ui::PerfUiPlugin;
use hex_grid::HexGrid;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(WindowPlugin {
				primary_window: Some(Window {
					title: "hex grid".into(),
					name: Some("hex-grid".into()),
					resolution: (1920.0, 1080.0).into(),
					resizable: false,
					enabled_buttons: bevy::window::EnabledButtons {
						maximize: false,
						..Default::default()
					},
					..default()
				}),
				..default()
			}),
			HexGrid,
			WireframePlugin,
			PanOrbitCameraPlugin,
			WorldInspectorPlugin::new(),
		))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
		.run();
}
