use arrayvec::ArrayVec;
use rend3::{resources::MaterialManager, types::TransparencyType, ModeData, RendererMode};
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, StencilState, TextureFormat, VertexState,
};

use crate::{
    common::{interfaces::ShaderInterfaces, shaders::mode_safe_shader},
    vertex::{cpu_vertex_buffers, gpu_vertex_buffers},
    SampleCount,
};

#[derive(Clone)]
pub struct BuildForwardPassShaderArgs<'a> {
    pub mode: RendererMode,
    pub device: &'a Device,

    pub interfaces: &'a ShaderInterfaces,

    pub directional_light_bgl: &'a BindGroupLayout,
    pub texture_bgl: ModeData<(), &'a BindGroupLayout>,

    pub materials: &'a MaterialManager,

    pub samples: SampleCount,
    pub transparency: TransparencyType,
}

pub fn build_forward_pass_shader(args: BuildForwardPassShaderArgs<'_>) -> RenderPipeline {
    let forward_pass_vert = unsafe {
        mode_safe_shader(
            args.device,
            args.mode,
            "forward pass vert",
            "opaque.vert.cpu.spv",
            "opaque.vert.gpu.spv",
            false,
        )
    };

    let forward_pass_frag = unsafe {
        mode_safe_shader(
            args.device,
            args.mode,
            "forward pass frag",
            "opaque.frag.cpu.spv",
            "opaque.frag.gpu.spv",
            false,
        )
    };

    let mut bgls: ArrayVec<&BindGroupLayout, 6> = ArrayVec::new();
    bgls.push(&args.interfaces.samplers_bgl);
    bgls.push(&args.interfaces.culled_object_bgl);
    bgls.push(args.directional_light_bgl);
    bgls.push(&args.interfaces.uniform_bgl);
    bgls.push(args.materials.get_bind_group_layout());
    if args.mode == RendererMode::GPUPowered {
        bgls.push(args.texture_bgl.as_gpu())
    }

    let pll = args.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("opaque pass"),
        bind_group_layouts: &bgls,
        push_constant_ranges: &[],
    });

    build_forward_pass_inner(args, pll, forward_pass_vert, forward_pass_frag)
}

fn build_forward_pass_inner(
    args: BuildForwardPassShaderArgs,
    pll: wgpu::PipelineLayout,
    forward_pass_vert: wgpu::ShaderModule,
    forward_pass_frag: wgpu::ShaderModule,
) -> RenderPipeline {
    let cpu_vertex_buffers = cpu_vertex_buffers();
    let gpu_vertex_buffers = gpu_vertex_buffers();

    args.device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(match args.transparency {
            TransparencyType::Opaque => "opaque pass",
            TransparencyType::Cutout => "cutout pass",
            TransparencyType::Blend => "blend forward pass",
        }),
        layout: Some(&pll),
        vertex: VertexState {
            module: &forward_pass_vert,
            entry_point: "main",
            buffers: match args.mode {
                RendererMode::CPUPowered => &cpu_vertex_buffers,
                RendererMode::GPUPowered => &gpu_vertex_buffers,
            },
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            clamp_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: match args.transparency {
                TransparencyType::Opaque | TransparencyType::Cutout => CompareFunction::Equal,
                TransparencyType::Blend => CompareFunction::GreaterEqual,
            },
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: args.samples as u32,
            ..Default::default()
        },
        fragment: Some(FragmentState {
            module: &forward_pass_frag,
            entry_point: "main",
            targets: &[ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: match args.transparency {
                    TransparencyType::Opaque | TransparencyType::Cutout => None,
                    TransparencyType::Blend => Some(BlendState::ALPHA_BLENDING),
                },
                write_mask: ColorWrites::all(),
            }],
        }),
    })
}
