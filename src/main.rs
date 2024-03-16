use bevy::prelude::*;
mod hex_grid;
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
					enabled_buttons: bevy::window::EnabledButtons{
						maximize: false,
						..Default::default()
					},
					..default()
				}),
				..default()
			}),
			HexGrid,
		))
		.run();
}
