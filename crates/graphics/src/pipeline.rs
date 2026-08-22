const SHADER: &str = r#"
struct VsOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-3.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var output: VsOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(_output: VsOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

pub(crate) fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("pet2d-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pet2d-pipeline-layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pet2d-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
