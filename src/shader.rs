use Error;

extern crate ash;

pub struct Shader<'a> {
	device: &'a ash::Device,

	vert_module: ash::vk::ShaderModule,
	frag_module: ash::vk::ShaderModule,

	pub vert_pipeline_layout: ash::vk::PipelineLayout,
	pub frag_pipeline_layout: ash::vk::PipelineLayout,

	pub pipeline: ash::vk::Pipeline,

	pub viewports: [ash::vk::Viewport; 1],
	pub scissors: [ash::vk::Rect2D; 1],
}

impl Shader<'_> {
	fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
		let mut cursor = std::io::Cursor::new(bytes);
		Ok(ash::util::read_spv(&mut cursor)?)
	}

	fn load_shader(device: &ash::Device, path: &str) -> Result<(
		ash::vk::ShaderModule,
		ash::vk::PipelineLayout,
	), Box<dyn Error>> {
		// read shader source

		let bytes = std::fs::read(path).expect("can't open file");
		let src = Self::read_shader_from_bytes(&bytes[..])?;

		// info & module

		let info = ash::vk::ShaderModuleCreateInfo::default().code(&src);

		let module = unsafe { device
			.create_shader_module(&info, None)
			.expect("can't create shader module") };

		// pipeline here

		let push_constant_range = ash::vk::PushConstantRange::default()
			.stage_flags(ash::vk::ShaderStageFlags::VERTEX)
			.offset(0)
			.size(64);

		let push_constant_ranges = &[
			vert_push_constant_range,
		];

		let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default()
			.push_constant_ranges(push_constant_ranges);

		let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

		Ok((
			module,
			pipeline_layout,
		))
	}

	pub fn new<'a>(device: &'a ash::Device, extent: ash::vk::Extent2D, renderpass: ash::vk::RenderPass, descriptor_set: u64, vert_path: &'a str, frag_path: &'a str) -> Result<Shader<'a>, Box<dyn Error>> {
		let (vert_module, vert_pipeline_layout) = Self::load_shader(device, vert_path).unwrap();
		let (frag_module, frag_pipeline_layout) = Self::load_shader(device, frag_path).unwrap();

		// shader entry

		let entry_name = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0") };

		let stage_create_infos = [
			ash::vk::PipelineShaderStageCreateInfo {
				// s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				module: vert_module,
				p_name: entry_name.as_ptr(),
				stage: ash::vk::ShaderStageFlags::VERTEX,
				..Default::default()
			},
			ash::vk::PipelineShaderStageCreateInfo {
				// s_type: ash::vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				module: frag_module,
				p_name: entry_name.as_ptr(),
				stage: ash::vk::ShaderStageFlags::FRAGMENT,
				..Default::default()
			},
		];

		// vertex input info

		let vertex_input_info = ash::vk::PipelineVertexInputStateCreateInfo { ..Default::default() };

		let vertex_input_assembly_info = ash::vk::PipelineInputAssemblyStateCreateInfo {
			topology: ash::vk::PrimitiveTopology::TRIANGLE_LIST,
			primitive_restart_enable: 0,
			..Default::default()
		};

		// viewports/scissor

		let viewports = [
			ash::vk::Viewport {
				x: 0.0,
				y: 0.0,
				width: extent.width as _,
				height: extent.height as _,
				min_depth: 0.0,
				max_depth: 1.0,
			},
		];

		let scissors = [
			ash::vk::Rect2D {
				offset: ash::vk::Offset2D { x: 0, y: 0 },
				extent,
			},
		];

		// other info
		// TODO check if these are already in defaults

		let viewport_info = ash::vk::PipelineViewportStateCreateInfo::default()
			.scissors(&scissors)
			.viewports(&viewports);

		let rasterization_info = ash::vk::PipelineRasterizationStateCreateInfo {
			polygon_mode: ash::vk::PolygonMode::FILL,
			line_width: 1.0,
			cull_mode: ash::vk::CullModeFlags::NONE,
			front_face: ash::vk::FrontFace::COUNTER_CLOCKWISE,
			depth_bias_constant_factor: 0.0,
			depth_bias_clamp: 0.0,
			depth_bias_slope_factor: 0.0,
			..Default::default()
		};

		let multisample_info = ash::vk::PipelineMultisampleStateCreateInfo {
			rasterization_samples: ash::vk::SampleCountFlags::TYPE_1,
			min_sample_shading: 1.0,
			..Default::default()
		};

		let noop_stencil_state = ash::vk::StencilOpState {
			fail_op: ash::vk::StencilOp::KEEP,
			pass_op: ash::vk::StencilOp::KEEP,
			depth_fail_op: ash::vk::StencilOp::KEEP,
			compare_op: ash::vk::CompareOp::ALWAYS,
			..Default::default()
		};

		let depth_info = ash::vk::PipelineDepthStencilStateCreateInfo {
			depth_test_enable: 1,
			depth_write_enable: 1,
			depth_compare_op: ash::vk::CompareOp::LESS_OR_EQUAL,
			front: noop_stencil_state,
			back: noop_stencil_state,
			max_depth_bounds: 1.0,
			..Default::default()
		};


		let color_blend_attachment_states = [
			ash::vk::PipelineColorBlendAttachmentState {
				color_write_mask: ash::vk::ColorComponentFlags::RGBA,
				blend_enable: 0,
				src_color_blend_factor: ash::vk::BlendFactor::ONE,
				dst_color_blend_factor: ash::vk::BlendFactor::ZERO,
				color_blend_op: ash::vk::BlendOp::ADD,
				src_alpha_blend_factor: ash::vk::BlendFactor::ONE,
				dst_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
				alpha_blend_op: ash::vk::BlendOp::ADD,
				..Default::default()
			}
		];

		let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default()
			.logic_op(ash::vk::LogicOp::CLEAR)
			.attachments(&color_blend_attachment_states);

		let dynamic_state = [ash::vk::DynamicState::VIEWPORT, ash::vk::DynamicState::SCISSOR];
		let dynamic_info = ash::vk::PipelineDynamicStateCreateInfo::default()
			.dynamic_states(&dynamic_state);

		let graphic_pipeline_info = ash::vk::GraphicsPipelineCreateInfo::default()
			.stages(&stage_create_infos)
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&vertex_input_assembly_info)
			.viewport_state(&viewport_info)
			.rasterization_state(&rasterization_info)
			.multisample_state(&multisample_info)
			.depth_stencil_state(&depth_info)
			.color_blend_state(&color_blend_state)
			.dynamic_state(&dynamic_info)
			.layout(vert_pipeline_layout)
			.render_pass(renderpass);



		let pipelines = unsafe { device
			.create_graphics_pipelines(ash::vk::PipelineCache::null(), &[graphic_pipeline_info], None)
			.expect("can't create graphics pipeline") };

		let pipeline = pipelines[0];

		// finally create pipeline

		Ok(Shader {
			device: device,

			vert_module: vert_module,
			frag_module: frag_module,

			vert_pipeline_layout: vert_pipeline_layout,
			frag_pipeline_layout: frag_pipeline_layout,

			pipeline: pipeline,

			viewports: viewports,
			scissors: scissors,
		})
	}
}

impl Drop for Shader<'_> {
	fn drop(&mut self) {
		unsafe {
			self.device.destroy_shader_module(self.vert_module, None);
			self.device.destroy_shader_module(self.frag_module, None);

			self.device.destroy_pipeline_layout(self.vert_pipeline_layout, None);
			self.device.destroy_pipeline_layout(self.frag_pipeline_layout, None);
		}
	}
}
