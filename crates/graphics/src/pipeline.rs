// Sprite pipeline: quads transformed by the Camera2D projection uniform.
// Group 0 carries the camera matrix; group 1 binds the texture and its
// sampler. Alpha blending composes transparent sprites over whatever was
// drawn (or cleared) before. The vertex tint multiplies the sampled texel;
// SpriteBatch collapses same-texture runs into single draw calls.

/// Bind group index of the camera uniform in the sprite pipeline.
pub(crate) const CAMERA_GROUP: u32 = 0;
/// Bind group index of the texture + sampler pair in the sprite pipeline.
pub(crate) const TEXTURE_GROUP: u32 = 1;

const SHADER: &str = r#"
@group(0) @binding(0) var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0) var t_sampler: sampler;
@group(1) @binding(1) var t_texture: texture_2d<f32>;

struct VsOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VsOutput {
    var output: VsOutput;
    output.position = camera * vec4<f32>(position, 0.0, 1.0);
    output.uv = uv;
    output.color = color;
    return output;
}

@fragment
fn fs_main(output: VsOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_texture, t_sampler, output.uv) * output.color;
    // premultiplied output: matches CompositeAlphaMode::PreMultiplied so
    // transparent windows (desktop pet) composite without edge halos; over
    // an opaque background the result is identical to straight "over"
    color = vec4(color.rgb * color.a, color.a);
    return color;
}
"#;

pub(crate) fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("pet2d-sprite-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pet2d-pipeline-layout"),
        bind_group_layouts: &[
            // group 0: camera uniform
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pet2d-camera-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
            // group 1: texture + sampler
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pet2d-texture-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            }),
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pet2d-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[crate::sprite::SpriteVertex::LAYOUT],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                // shader premultiplies first (see fs_main); this keeps the
                // framebuffer valid for PreMultiplied surface compositing
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
