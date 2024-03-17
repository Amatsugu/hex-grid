use bevy::{
	pbr::wireframe::WireframeConfig,
	prelude::*,
	render::{
		mesh::{Indices, PrimitiveTopology},
		render_asset::RenderAssetUsages,
		render_resource::{Extent3d, TextureDimension, TextureFormat},
	},
};
use bevy_panorbit_camera::PanOrbitCamera;

pub struct HexGrid;

pub struct HexCoord {
	pub pos: Vec3,
}

const WIREFRAME: bool = true;
const OUTER_RADIUS: f32 = 1.;
const INNER_RADIUS: f32 = OUTER_RADIUS * 0.866025404;
const HEX_CORNERS: [Vec3; 7] = [
	Vec3::new(0., 0., OUTER_RADIUS),
	Vec3::new(INNER_RADIUS, 0., 0.5 * OUTER_RADIUS),
	Vec3::new(INNER_RADIUS, 0., -0.5 * OUTER_RADIUS),
	Vec3::new(0., 0., -OUTER_RADIUS),
	Vec3::new(-INNER_RADIUS, 0., -0.5 * OUTER_RADIUS),
	Vec3::new(-INNER_RADIUS, 0., 0.5 * OUTER_RADIUS),
	Vec3::new(0., 0., OUTER_RADIUS),
];

impl Plugin for HexGrid {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, (create_hex_grid, setup));
		if WIREFRAME {
			app.insert_resource(WireframeConfig {
				global: true,
				default_color: Color::WHITE,
			});
		}
	}
}

fn setup(mut commands: Commands) {
	let camera_and_light_transform =
		Transform::from_xyz(0., 50., 0.).looking_at(Vec3::new(50., 0., 50.), Vec3::Y);

	commands.spawn((
		Camera3dBundle {
			transform: camera_and_light_transform,
			..default()
		},
		PanOrbitCamera::default(),
	));

	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0.0, 16.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

fn create_hex_grid(
	mut commands: Commands,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut images: ResMut<Assets<Image>>,
	mut meshes: ResMut<Assets<Mesh>>,
) {
	let debug_material = materials.add(StandardMaterial {
		base_color_texture: Some(images.add(uv_debug_texture())),
		..default()
	});

	let count = 100 * 3 * 6;
	let mut verts = Vec::with_capacity(count);
	let mut uvs = Vec::with_capacity(count);
	let mut normals = Vec::with_capacity(count);
	let mut indices = Vec::with_capacity(count);

	for z in 0..10 {
		for x in 0..10 {
			let off_pos = Vec3::new(x as f32, x as f32 * 1., z as f32);
			let grid_pos = to_hex_pos(off_pos);
			create_tile(grid_pos, &mut verts, &mut uvs, &mut normals, &mut indices);
		}
	}
	let mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
	)
	.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
	.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
	.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
	.with_inserted_indices(Indices::U32(indices));
	commands.spawn(PbrBundle {
		mesh: meshes.add(mesh),
		material: debug_material,
		..default()
	});
}

fn to_hex_pos(pos: Vec3) -> Vec3 {
	let x = (pos.x + pos.z * 0.5 - (pos.z / 2.).floor()) * (INNER_RADIUS * 2.);
	return Vec3::new(x, pos.y, pos.z * OUTER_RADIUS * 1.5);
}

fn create_tile(
	pos: Vec3,
	verts: &mut Vec<Vec3>,
	uvs: &mut Vec<Vec2>,
	normals: &mut Vec<Vec3>,
	indices: &mut Vec<u32>,
) {
	let idx = verts.len() as u32;
	// [v0,v1,v2,v3,v4,v5,v6]
	normals.push(Vec3::Y);
	uvs.push(pos.normalize().xz());
	verts.push(pos);
	for i in 0..6 {
		verts.push(pos + HEX_CORNERS[i]);
		uvs.push((pos + HEX_CORNERS[i]).normalize().xz());
		normals.push(Vec3::Y);
		indices.push(idx);
		indices.push(idx + 1 + i as u32);
		indices.push(idx + 1 + ((i as u32 + 1) % 6));
	}
}

fn uv_debug_texture() -> Image {
	const TEXTURE_SIZE: usize = 8;

	let mut palette: [u8; 32] = [
		255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
		198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
	];

	let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
	for y in 0..TEXTURE_SIZE {
		let offset = TEXTURE_SIZE * y * 4;
		texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
		palette.rotate_right(4);
	}

	Image::new_fill(
		Extent3d {
			width: TEXTURE_SIZE as u32,
			height: TEXTURE_SIZE as u32,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		&texture_data,
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD,
	)
}
