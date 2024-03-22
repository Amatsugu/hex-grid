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

extern crate rand;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

pub struct HexGrid;

const MAP_SIZE: u32 = 64;
const WIREFRAME: bool = false;
const OUTER_RADIUS: f32 = 1.;
const INNER_RADIUS: f32 = OUTER_RADIUS * 0.866025404;
const CHUNK_SIZE: u32 = 32;
const HEX_CORNERS: [Vec3; 6] = [
	Vec3::new(0., 0., OUTER_RADIUS),
	Vec3::new(INNER_RADIUS, 0., 0.5 * OUTER_RADIUS),
	Vec3::new(INNER_RADIUS, 0., -0.5 * OUTER_RADIUS),
	Vec3::new(0., 0., -OUTER_RADIUS),
	Vec3::new(-INNER_RADIUS, 0., -0.5 * OUTER_RADIUS),
	Vec3::new(-INNER_RADIUS, 0., 0.5 * OUTER_RADIUS),
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
		transform: Transform::from_xyz(0.0, 16.0, 0.).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

fn draw_gizmos(mut gizmos: Gizmos) {
	gizmos.arrow(Vec3::ZERO, Vec3::Y * 1.5, Color::GREEN);
	gizmos.arrow(Vec3::ZERO, Vec3::Z * 1.5, Color::BLUE);
	gizmos.arrow(Vec3::ZERO, Vec3::X * 1.5, Color::RED);

	for i in 0..6 {
		gizmos.arrow(
			HEX_CORNERS[i],
			HEX_CORNERS[i] + Vec3::Y * (i + 1) as f32,
			Color::ALICE_BLUE,
		);
	}
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
	for z in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let pos = to_hex_pos(Vec3::new(x as f32, 0., z as f32) * CHUNK_SIZE as f32);
			let mesh = create_chunk();
			commands.spawn(PbrBundle {
				mesh: meshes.add(mesh),
				material: debug_material.clone(),
				transform: Transform::from_translation(pos),
				..default()
			});
		}
	}
}

fn create_chunk() -> Mesh {
	let count = (CHUNK_SIZE * CHUNK_SIZE * 3 * 6) as usize;
	let mut verts = Vec::with_capacity(count);
	let mut uvs = Vec::with_capacity(count);
	let mut normals = Vec::with_capacity(count);
	let mut indices = Vec::with_capacity(count);
	let mut rng = ChaCha20Rng::seed_from_u64(2);

	for z in 0..CHUNK_SIZE {
		for x in 0..CHUNK_SIZE {
			let off_pos = Vec3::new(x as f32, rng.gen_range(0..3) as f32, z as f32);
			let grid_pos = to_hex_pos(off_pos);
			create_tile(grid_pos, &mut verts, &mut uvs, &mut normals, &mut indices);
		}
	}
	for z in 0..CHUNK_SIZE {
		for x in 0..CHUNK_SIZE {
			let idx = ((x * 7) + (z * CHUNK_SIZE * 7)) as u32;
			add_tile_sides(x, z, idx, &mut indices, &verts);
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
	return mesh;
}

fn to_hex_pos(pos: Vec3) -> Vec3 {
	let x = (pos.x + pos.z * 0.5 - (pos.z / 2.).floor()) * (INNER_RADIUS * 2.);
	return Vec3::new(x, pos.y, pos.z * OUTER_RADIUS * 1.5);
}

fn add_tile_sides(x: u32, z: u32, idx: u32, indices: &mut Vec<u32>, verts: &Vec<Vec3>) {
	let c_tile = idx + 1;
	const TILE_WIDTH: u32 = 7;
	const ROW_WIDTH: u32 = CHUNK_SIZE * TILE_WIDTH;

	if x < CHUNK_SIZE - 1 {
		let n_tile = c_tile + TILE_WIDTH;
		create_quad(
			c_tile + 1,
			c_tile + 2,
			n_tile + 4,
			n_tile + 5,
			indices,
			verts,
		);
	}

	if z < CHUNK_SIZE - 1 {
		if z % 2 == 0 {
			let d_tile = c_tile + ROW_WIDTH;
			create_quad(c_tile, c_tile + 1, d_tile + 3, d_tile + 4, indices, verts);
		} else if x < CHUNK_SIZE - 1 {
			let d_tile = c_tile + ROW_WIDTH + TILE_WIDTH;
			create_quad(c_tile, c_tile + 1, d_tile + 3, d_tile + 4, indices, verts);
		}
	}

	if x > 0 && z % 2 == 0 {
		let d_tile = c_tile + ROW_WIDTH - TILE_WIDTH;
		create_quad(c_tile + 5, c_tile, d_tile + 2, d_tile + 3, indices, verts);
	}
	if z % 2 == 1 && z < CHUNK_SIZE - 1 {
		let d_tile = c_tile + ROW_WIDTH;
		create_quad(c_tile + 5, c_tile, d_tile + 2, d_tile + 3, indices, verts);
	}
}

fn create_quad(v1: u32, v2: u32, v3: u32, v4: u32, indices: &mut Vec<u32>, verts: &Vec<Vec3>) {
	if verts[v1 as usize].y == verts[v3 as usize].y {
		return;
	}
	indices.push(v1);
	indices.push(v3);
	indices.push(v2);

	indices.push(v1);
	indices.push(v4);
	indices.push(v3);
}

fn create_tile(
	pos: Vec3,
	verts: &mut Vec<Vec3>,
	uvs: &mut Vec<Vec2>,
	normals: &mut Vec<Vec3>,
	indices: &mut Vec<u32>,
) {
	let idx = verts.len() as u32;
	normals.push(Vec3::Y);
	uvs.push(pos.xz());
	verts.push(pos);
	for i in 0..6 {
		verts.push(pos + HEX_CORNERS[i]);
		uvs.push((pos + HEX_CORNERS[i]).xz());
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
