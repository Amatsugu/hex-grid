use bevy::{
	pbr::{wireframe::WireframeConfig, CascadeShadowConfig, DirectionalLightShadowMap},
	prelude::*,
	render::{
		mesh::{Indices, PrimitiveTopology},
		render_asset::RenderAssetUsages,
		render_resource::{Extent3d, TextureDimension, TextureFormat},
	},
};
use bevy_panorbit_camera::PanOrbitCamera;

use bevy_inspector_egui::prelude::*;

use noise::{NoiseFn, SuperSimplex};

pub struct HexGrid;

const MAP_SIZE: u32 = 32;
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
		app.add_systems(Startup, (create_hex_grid, setup))
			.add_systems(Update, draw_gizmos)
			.insert_resource(DirectionalLightShadowMap { size: 2048 });
		if WIREFRAME {
			app.insert_resource(WireframeConfig {
				global: true,
				default_color: Color::WHITE,
			});
		}
	}
}

fn setup(mut commands: Commands) {
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(0., 50., 0.)
				.looking_at(Vec3::new(50., 0., 50.), Vec3::Y),
			..default()
		},
		PanOrbitCamera {
			..Default::default()
		},
	));

	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadows_enabled: false,
			..default()
		},
		cascade_shadow_config: CascadeShadowConfig {
			bounds: vec![20., 40., 80., 1000., 5000., 19000., 20000.],
			..default()
		},
		transform: Transform::from_xyz(0.0, 16.0, 5.).looking_at(Vec3::ZERO, Vec3::Y),
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
		// base_color_texture: Some(images.add(uv_debug_texture())),
		..default()
	});

	let noise = SuperSimplex::new(1223939298);
	for z in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let pos = to_hex_pos(Vec3::new(x as f32, 0., z as f32) * CHUNK_SIZE as f32);
			let mesh = create_chunk(x, z, &noise);
			commands.spawn(PbrBundle {
				mesh: meshes.add(mesh),
				material: debug_material.clone(),
				transform: Transform::from_translation(pos),
				..default()
			});
		}
	}
}

fn create_chunk(c_x: u32, c_z: u32, noise: &SuperSimplex) -> Mesh {
	const COUNT: usize = (CHUNK_SIZE * CHUNK_SIZE * 3 * 6) as usize;
	let mut verts = Vec::with_capacity(COUNT);
	let mut uvs = Vec::with_capacity(COUNT);
	let mut normals = Vec::with_capacity(COUNT);
	let mut indices = Vec::with_capacity(COUNT);

	for z in 0..CHUNK_SIZE {
		for x in 0..CHUNK_SIZE {
			let height = sample_height(x + c_x * CHUNK_SIZE, z + c_z * CHUNK_SIZE, noise);
			let off_pos = Vec3::new(x as f32, height, z as f32);
			let grid_pos = to_hex_pos(off_pos);
			create_tile(grid_pos, &mut verts, &mut uvs, &mut normals, &mut indices);
		}
	}
	for z in 0..CHUNK_SIZE {
		for x in 0..CHUNK_SIZE {
			let idx = ((x * 7) + (z * CHUNK_SIZE * 7)) as u32;
			add_tile_sides(x, z, idx, &mut indices, &mut normals, &verts);
		}
	}

	add_chunk_sides(
		c_x,
		c_z,
		&mut verts,
		&mut indices,
		&mut normals,
		&mut uvs,
		noise,
	);

	let mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
	)
	.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
	.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
	// .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
	.with_inserted_indices(Indices::U32(indices))
	.with_duplicated_vertices()
	.with_computed_flat_normals();
	return mesh;
}

fn to_hex_pos(pos: Vec3) -> Vec3 {
	let x = (pos.x + pos.z * 0.5 - (pos.z / 2.).floor()) * (INNER_RADIUS * 2.);
	return Vec3::new(x, pos.y, pos.z * OUTER_RADIUS * 1.5);
}

fn add_chunk_sides(
	c_x: u32,
	c_z: u32,
	verts: &mut Vec<Vec3>,
	indices: &mut Vec<u32>,
	normals: &mut Vec<Vec3>,
	uvs: &mut Vec<Vec2>,
	noise: &SuperSimplex,
) {
	if c_x < MAP_SIZE - 1 {
		//draw top side
		let x = CHUNK_SIZE - 1;
		for z in 0..CHUNK_SIZE {
			let c_tile = ((x * 7) + (z * 7 * CHUNK_SIZE)) as u32 + 1;
			let mut height = sample_height(x + 1 + c_x * CHUNK_SIZE, z + c_z * CHUNK_SIZE, noise);
			let mut off_pos = Vec3::new(x as f32, height, z as f32);
			let mut grid_pos = to_hex_pos(off_pos);
			let mut center = Vec3::new(grid_pos.x, 0., grid_pos.z);

			let mut idx = verts.len() as u32;

			let mut p = grid_pos + HEX_CORNERS[2];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());

			p = grid_pos + HEX_CORNERS[1];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());
			create_quad(c_tile + 1, c_tile + 2, idx, idx + 1, indices, verts);

			if z % 2 == 1 {
				if z > 0 {
					height =
						sample_height(x + 1 + c_x * CHUNK_SIZE, z - 1 + c_z * CHUNK_SIZE, noise);
					off_pos = Vec3::new(x as f32, height, z as f32);
					grid_pos = to_hex_pos(off_pos);
					center = Vec3::new(grid_pos.x, 0., grid_pos.z);

					idx = verts.len() as u32;
					p = grid_pos + HEX_CORNERS[2];
					verts.push(p);
					uvs.push(p.xz() / CHUNK_SIZE as f32);
					normals.push((p - center).normalize());

					p = grid_pos + HEX_CORNERS[3];
					verts.push(p);
					uvs.push(p.xz() / CHUNK_SIZE as f32);
					normals.push((p - center).normalize());

					create_quad(c_tile + 2, c_tile + 3, idx + 1, idx, indices, verts);
				}
				if z < CHUNK_SIZE - 1 {
					height =
						sample_height(x + 1 + c_x * CHUNK_SIZE, z + 1 + c_z * CHUNK_SIZE, noise);
					off_pos = Vec3::new(x as f32, height, z as f32);
					grid_pos = to_hex_pos(off_pos);
					center = Vec3::new(grid_pos.x, 0., grid_pos.z);

					idx = verts.len() as u32;
					p = grid_pos + HEX_CORNERS[0];
					verts.push(p);
					uvs.push(p.xz() / CHUNK_SIZE as f32);
					normals.push((p - center).normalize());

					p = grid_pos + HEX_CORNERS[1];
					verts.push(p);
					uvs.push(p.xz() / CHUNK_SIZE as f32);
					normals.push((p - center).normalize());

					create_quad(c_tile, c_tile + 1, idx + 1, idx, indices, verts);
				}
			}
		}
	}
	if c_z < MAP_SIZE - 1 {
		//draw right side
		let z = CHUNK_SIZE - 1;
		for x in 0..CHUNK_SIZE {
			let c_tile = ((x * 7) + (z * 7 * CHUNK_SIZE)) as u32 + 1;
			let mut height = sample_height(x + c_x * CHUNK_SIZE, z + 1 + c_z * CHUNK_SIZE, noise);
			let mut off_pos = Vec3::new(x as f32, height, z as f32);
			let mut grid_pos = to_hex_pos(off_pos);
			let mut center = Vec3::new(grid_pos.x, 0., grid_pos.z);

			let idx = verts.len() as u32;

			let mut p = grid_pos + HEX_CORNERS[0];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());

			p = grid_pos + HEX_CORNERS[5];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());
			create_quad(c_tile + 5, c_tile, idx, idx + 1, indices, verts);

			height = sample_height(x + 1 + c_x * CHUNK_SIZE, z + 1 + c_z * CHUNK_SIZE, noise);
			off_pos = Vec3::new(x as f32, height, z as f32);
			grid_pos = to_hex_pos(off_pos);
			center = Vec3::new(grid_pos.x, 0., grid_pos.z);

			p = grid_pos + HEX_CORNERS[0];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());

			p = grid_pos + HEX_CORNERS[1];
			verts.push(p);
			uvs.push(p.xz() / CHUNK_SIZE as f32);
			normals.push((p - center).normalize());
			create_quad(c_tile, c_tile + 1, idx + 3, idx + 2, indices, verts);
		}
	}
}

fn add_tile_sides(
	x: u32,
	z: u32,
	idx: u32,
	indices: &mut Vec<u32>,
	normals: &mut Vec<Vec3>,
	verts: &Vec<Vec3>,
) {
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
	let vert1 = verts[v1 as usize];
	let vert3 = verts[v3 as usize];
	if vert1.y == vert3.y {
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
	let center = Vec3::new(pos.x, 0., pos.z);
	normals.push(Vec3::Y);
	uvs.push(pos.xz() / CHUNK_SIZE as f32);
	verts.push(pos);
	for i in 0..6 {
		let p = pos + HEX_CORNERS[i];
		verts.push(p);
		uvs.push(p.xz() / CHUNK_SIZE as f32);
		normals.push((p - center).normalize());
		indices.push(idx);
		indices.push(idx + 1 + i as u32);
		indices.push(idx + 1 + ((i as u32 + 1) % 6));
	}
}

const NOISE_SCALE: f64 = 350.;
const SEA_LEVEL: f64 = 5.;

fn sample_height(x: u32, y: u32, noise: &SuperSimplex) -> f32 {
	let mut elevation = 0.;

	let x_s = x as f64 / NOISE_SCALE;
	let y_s = y as f64 / NOISE_SCALE;

	let first_layer = sample_layer(noise, x_s, y_s, 2.14, 0.87, 0.77, -0.2, 2.93, 4);
	elevation += first_layer;
	elevation += sample_layer(noise, x_s, y_s, 2.85, 2., 1., 0., -0.23, 4);
	elevation += mask(
		first_layer,
		sample_layer_rigid(noise, x_s, y_s, 2.6, 4., 1.57, 0., 10.44, 0.35, 4),
	);
	elevation += mask(
		first_layer,
		sample_layer_rigid(noise, x_s, y_s, 3.87, 5.8, 0., 0., -1., 4.57, 3),
	);

	return elevation as f32;
}

fn mask(first_layer: f64, value: f64) -> f64 {
	let mask = (first_layer - SEA_LEVEL).max(0.);
	return value * mask;
}

fn sample_layer(
	noise: &SuperSimplex,
	x: f64,
	z: f64,
	base_roughness: f64,
	roughness: f64,
	persistence: f64,
	min_value: f64,
	strength: f64,
	layers: usize,
) -> f64 {
	let mut freq: f64 = base_roughness;
	let mut amp: f64 = 1.;
	let mut value = 0.;

	for _ in 0..layers {
		let v = noise.get([x * freq, z * freq]);
		value += (v + 1.) * 0.5 * amp;
		freq *= roughness;
		amp *= persistence;
	}
	value -= min_value;
	return value * strength;
}

fn sample_layer_rigid(
	noise: &SuperSimplex,
	x: f64,
	z: f64,
	base_roughness: f64,
	roughness: f64,
	persistence: f64,
	min_value: f64,
	strength: f64,
	weight_multi: f64,
	layers: usize,
) -> f64 {
	let mut freq: f64 = base_roughness;
	let mut amp: f64 = 1.;
	let mut value = 0.;
	let mut weight = 1.;
	for _ in 0..layers {
		let mut v = 1. - noise.get([x * freq, z * freq]).abs();
		v *= v;
		v *= weight;
		weight = v * weight_multi;
		weight = weight.clamp(0., 1.);
		value += v * amp;
		freq *= roughness;
		amp *= persistence;
	}
	value -= min_value;
	return value * strength;
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
