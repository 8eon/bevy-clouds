use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, TextureDimension, TextureFormat},
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<CloudMaterial>::default())
        .add_systems(Startup, setup)
        .run();
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1, dimension = "3d")]
    #[sampler(2)]
    pub noise_texture: Handle<Image>,
}

impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cloud_shader.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // --- 3D Worley Noise Generation ---
    // Worley noise (cellular noise) gives that "billowy" look.
    let size = 32; // Reduced resolution for performance and RAM
    let mut data = Vec::with_capacity(size * size * size);
    
    // Generate random feature points for Worley noise
    let num_points = 16;
    let mut rng = rand::thread_rng();
    let mut points = Vec::new();
    for _ in 0..num_points {
        points.push(Vec3::new(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        ));
    }

    for z in 0..size {
        let fz = z as f32 / size as f32;
        for y in 0..size {
            let fy = y as f32 / size as f32;
            for x in 0..size {
                let fx = x as f32 / size as f32;
                let p = Vec3::new(fx, fy, fz);
                
                // Find distance to closest point (with simple wrapping for tiling)
                let mut min_dist = 1.0;
                for point in &points {
                    // Simple distance check (not perfectly tiling yet but good for a start)
                    let dist = p.distance(*point);
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                
                // Invert and scale for cloud density (1.0 at center, 0.0 at edges)
                let val = (1.0 - (min_dist * 2.5).min(1.0)) * 255.0;
                data.push(val as u8);
            }
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: size as u32,
        },
        TextureDimension::D3,
        data,
        TextureFormat::R8Unorm,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    let noise_handle = images.add(image);

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
    ));

    // Cloud Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(cloud_materials.add(CloudMaterial {
            color: LinearRgba::from(Color::srgb(0.9, 0.9, 1.0)),
            noise_texture: noise_handle,
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            range: 20.0,
            intensity: 5000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}
