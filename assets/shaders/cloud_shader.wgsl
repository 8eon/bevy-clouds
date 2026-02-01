#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::mesh_bindings as mesh_bindings
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_functions as mesh_functions

struct CloudMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: CloudMaterial;
@group(2) @binding(1)
var noise_texture: texture_3d<f32>;
@group(2) @binding(2)
var noise_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_matrix = mesh_functions::get_world_from_local(0u);
    let world_pos4 = mesh_functions::mesh_position_local_to_world(world_matrix, vec4<f32>(vertex.position, 1.0));
    out.world_position = world_pos4;
    out.position = view_bindings::view.clip_from_world * world_pos4;
    return out;
}

fn ray_box_intersection(ray_origin: vec3<f32>, ray_dir: vec3<f32>, box_min: vec3<f32>, box_max: vec3<f32>) -> vec2<f32> {
    let inv_dir = 1.0 / ray_dir;
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    let dist_a = max(max(tmin.x, tmin.y), tmin.z);
    let dist_b = min(min(tmax.x, tmax.y), tmax.z);
    return vec2<f32>(dist_a, dist_b);
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let ray_origin = view_bindings::view.world_position;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);

    let box_min = vec3<f32>(-1.0, 0.0, -1.0);
    let box_max = vec3<f32>(1.0, 2.0, 1.0);

    let t = ray_box_intersection(ray_origin, ray_dir, box_min, box_max);
    let t_entry = max(t.x, 0.0); 
    let t_exit = t.y;

    if (t_entry < t_exit) {
        var p = ray_origin + ray_dir * t_entry;
        var total_transmittance = 1.0;
        var final_color = vec3<f32>(0.0);
        
        // --- PERFORMANCE OPTIMIZATION ---
        // Reduced steps back to 32 as requested
        let steps = 32; 
        let step_size = (t_exit - t_entry) / f32(steps);
        let absorption = 3.0; // Adjusted for lower steps

        for (var i = 0; i < steps; i = i + 1) {
            let uv = (p - box_min) / (box_max - box_min);
            let noise_val = textureSampleLevel(noise_texture, noise_sampler, uv, 0.0).r;
            
            // Worley noise thresholding
            let density = max(noise_val - 0.1, 0.0) * 2.0;
            
            if (density > 0.0) {
                let step_transmittance = exp(-density * step_size * absorption);
                
                let height_factor = (p.y - box_min.y) / (box_max.y - box_min.y);
                let light = mix(0.5, 1.0, height_factor);
                
                let ambient = material.color.rgb * light;
                
                final_color += total_transmittance * (1.0 - step_transmittance) * ambient;
                total_transmittance *= step_transmittance;
            }

            if (total_transmittance <= 0.05) { // Early exit threshold
                break;
            }
            
            p += ray_dir * step_size;
        }

        return vec4<f32>(final_color, 1.0 - total_transmittance);
    } else {
        discard;
    }
}
