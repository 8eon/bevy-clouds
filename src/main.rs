use bevy::{
    prelude::*,
    input::mouse::MouseMotion,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

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
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.9, 0.9, 1.0),
            density_multiplier: 2.0,
            threshold: 0.2,
            absorption: 3.0,
            steps: 16,
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
    // Cloud Cube
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

    // Simple Orbit Camera
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

        if ui.button("Reset").clicked() {
            *settings = CloudSettings::default();
        }
    });
}

fn update_material_system(
    settings: Res<CloudSettings>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
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
