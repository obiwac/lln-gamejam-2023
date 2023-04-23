use Error;
use Context;

extern crate ash;

pub struct Shader<'a> {
	context: &'a Context<'a>,

	vert_module: ash::vk::ShaderModule,
	frag_module: ash::vk::ShaderModule,

	vert_pipeline_layout: ash::vk::PipelineLayout,
	frag_pipeline_layout: ash::vk::PipelineLayout,
}

impl Shader<'_> {
	fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
		let mut cursor = std::io::Cursor::new(bytes);
		Ok(ash::util::read_spv(&mut cursor)?)
	}

	fn load_shader(context: &Context, path: &str) -> Result<(
		ash::vk::ShaderModule,
		ash::vk::PipelineLayout,
	), Box<dyn Error>> {
		// read shader source

		let bytes = std::fs::read(path).expect("can't open file");
		let src = Self::read_shader_from_bytes(&bytes[..])?;

		// info & module

		let info = ash::vk::ShaderModuleCreateInfo::default().code(&src);

		let module = unsafe { context.device
			.create_shader_module(&info, None)
			.expect("can't create shader module") };

		// pipeline here

		let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default();
		let pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None)? };

		Ok((
			module,
			pipeline_layout,
		))
	}

	pub fn new<'a>(context: &'a Context<'a>, vert_path: &'a str, frag_path: &'a str) -> Result<Shader<'a>, Box<dyn Error>> {
		let (vert_module, vert_pipeline_layout) = Self::load_shader(context, vert_path).unwrap();
		let (frag_module, frag_pipeline_layout) = Self::load_shader(context, frag_path).unwrap();

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

		Ok(Shader {
			context: context,

			vert_module: vert_module,
			frag_module: frag_module,

			vert_pipeline_layout: vert_pipeline_layout,
			frag_pipeline_layout: frag_pipeline_layout,
		})
	}
}

impl Drop for Shader<'_> {
	fn drop(&mut self) {
		unsafe {
			self.context.device.destroy_shader_module(self.vert_module, None);
			self.context.device.destroy_shader_module(self.frag_module, None);

			self.context.device.destroy_pipeline_layout(self.vert_pipeline_layout, None);
			self.context.device.destroy_pipeline_layout(self.frag_pipeline_layout, None);
		}
	}
}
