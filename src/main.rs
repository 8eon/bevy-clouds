use bevy::{
    prelude::*,
    input::mouse::MouseMotion,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType, TextureDimension, TextureFormat},
    render::render_asset::RenderAssetUsages,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(MaterialPlugin::<CloudMaterial>::default())
        .init_resource::<CloudSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_control_system, ui_system, update_material_system))
        .run();
}

#[derive(Resource)]
pub struct CloudSettings {
    pub color: Color,
    pub density_multiplier: f32,
    pub threshold: f32,
    pub absorption: f32,
    pub steps: u32,
    pub seed: u32,
    pub frequency: f32,
    pub cell_count: u32, // New setting for cell density
    pub noise_handle: Handle<Image>,
    pub needs_rebuild: bool,
}

impl FromWorld for CloudSettings {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let size = 32;
        let image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: size,
            },
            TextureDimension::D3,
            &[0],
            TextureFormat::R8Unorm,
            RenderAssetUsages::default(),
        );
        let noise_handle = images.add(image);

        Self {
            color: Color::srgb(0.9, 0.9, 1.0),
            density_multiplier: 2.0,
            threshold: 0.2,
            absorption: 3.0,
            steps: 16,
            seed: 1,
            frequency: 4.0,
            cell_count: 16,
            noise_handle,
            needs_rebuild: true,
        }
    }
}

#[derive(Component)]
struct OrbitCamera {
    pub center: Vec3,
    pub distance: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    #[uniform(0)]
    pub data: CloudMaterialUniform,
    #[texture(1, dimension = "3d")]
    #[sampler(2)]
    pub noise_texture: Handle<Image>,
}

#[derive(ShaderType, Debug, Clone)]
pub struct CloudMaterialUniform {
    pub color: LinearRgba,
    pub settings: Vec4, // x: density, y: threshold, z: absorption, w: steps
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
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    settings: Res<CloudSettings>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(cloud_materials.add(CloudMaterial {
            data: CloudMaterialUniform {
                color: LinearRgba::from(settings.color),
                settings: Vec4::new(
                    settings.density_multiplier,
                    settings.threshold,
                    settings.absorption,
                    settings.steps as f32,
                ),
            },
            noise_texture: settings.noise_handle.clone(),
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            range: 20.0,
            intensity: 5000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 7.0,
        },
    ));
}

fn ui_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<CloudSettings>,
) {
    egui::Window::new("Cloud Settings").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut settings.density_multiplier, 0.0..=10.0).text("Density"));
        ui.add(egui::Slider::new(&mut settings.threshold, 0.0..=1.0).text("Threshold"));
        ui.add(egui::Slider::new(&mut settings.absorption, 0.0..=10.0).text("Absorption"));
        
        let mut steps_f32 = settings.steps as f32;
        ui.add(egui::Slider::new(&mut steps_f32, 4.0..=64.0).text("Steps"));
        settings.steps = steps_f32 as u32;

        ui.separator();
        ui.label("Noise Generation (CPU Bake)");
        if ui.add(egui::Slider::new(&mut settings.seed, 0..=100).text("Seed")).changed() {
            settings.needs_rebuild = true;
        }
        if ui.add(egui::Slider::new(&mut settings.frequency, 1.0..=10.0).text("Frequency")).changed() {
            settings.needs_rebuild = true;
        }
        if ui.add(egui::Slider::new(&mut settings.cell_count, 4..=64).text("Cell Count")).changed() {
            settings.needs_rebuild = true;
        }

        if ui.button("Reset").clicked() {
            settings.color = Color::srgb(0.9, 0.9, 1.0);
            settings.density_multiplier = 2.0;
            settings.threshold = 0.2;
            settings.absorption = 3.0;
            settings.steps = 16;
            settings.seed = 1;
            settings.frequency = 4.0;
            settings.cell_count = 16;
            settings.needs_rebuild = true;
        }
    });
}

fn update_material_system(
    mut settings: ResMut<CloudSettings>,
    mut materials: ResMut<Assets<CloudMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if settings.needs_rebuild {
        if let Some(image) = images.get_mut(&settings.noise_handle) {
            let size = 32;
            let mut data = Vec::with_capacity(size * size * size);
            
            let mut rng = ChaCha8Rng::seed_from_u64(settings.seed as u64);
            let num_points = settings.cell_count as usize;
            let mut points = Vec::new();
            for _ in 0..num_points {
                points.push(Vec3::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ));
            }

            let freq = settings.frequency;
            for z in 0..size {
                let fz = z as f32 / size as f32;
                for y in 0..size {
                    let fy = y as f32 / size as f32;
                    for x in 0..size {
                        let fx = x as f32 / size as f32;
                        let p = Vec3::new(fx, fy, fz) * freq;
                        
                        let mut min_dist = 10.0;
                        for point in &points {
                            // Simple tiling logic for better billows
                            for oz in -1..=1 {
                                for oy in -1..=1 {
                                    for ox in -1..=1 {
                                        let offset = Vec3::new(ox as f32, oy as f32, oz as f32);
                                        let dist = p.distance((*point + offset) * freq);
                                        if dist < min_dist {
                                            min_dist = dist;
                                        }
                                    }
                                }
                            }
                        }
                        let val = (1.0 - min_dist.min(1.0)) * 255.0;
                        data.push(val as u8);
                    }
                }
            }
            image.data = data;
            settings.needs_rebuild = false;
        }
    }

    for (_, material) in materials.iter_mut() {
        material.data.color = LinearRgba::from(settings.color);
        material.data.settings = Vec4::new(
            settings.density_multiplier,
            settings.threshold,
            settings.absorption,
            settings.steps as f32,
        );
    }
}

fn camera_control_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let (orbit, mut transform) = query.single_mut();
    
    if buttons.pressed(MouseButton::Left) {
        for event in mouse_motion_events.read() {
            let delta_x = event.delta.x * 0.005;
            let delta_y = event.delta.y * 0.005;
            
            let mut angles = transform.rotation.to_euler(EulerRot::YXZ);
            angles.0 -= delta_x;
            angles.1 -= delta_y;
            angles.1 = angles.1.clamp(-1.5, 1.5);
            
            transform.rotation = Quat::from_euler(EulerRot::YXZ, angles.0, angles.1, 0.0);
        }
    } else {
        mouse_motion_events.clear();
    }
    
    let rot_matrix = Mat3::from_quat(transform.rotation);
    transform.translation = orbit.center + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, orbit.distance));
}
