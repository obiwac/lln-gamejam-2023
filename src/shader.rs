use Error;
use Context;

extern crate ash;

pub struct Shader<'a> {
	context: &'a Context<'a>,
	pipeline_layout: ash::vk::PipelineLayout,
}

impl Shader<'_> {
	fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
		let mut cursor = std::io::Cursor::new(bytes);
		Ok(ash::util::read_spv(&mut cursor)?)
	}

	pub fn new<'a>(context: &'a Context<'a>, vert_path: &'a str, frag_path: &'a str) -> Result<Shader<'a>, Box<dyn Error>> {
		// read shader source

		let bytes = std::fs::read(vert_path).expect("can't open file");
		let src = Self::read_shader_from_bytes(&bytes[..])?;

		// info & module

		let info = ash::vk::ShaderModuleCreateInfo::default().code(&src);

		let module = unsafe { context.device
			.create_shader_module(&info, None)
			.expect("can't create shader module") };

		// pipeline here

		let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default();
		let pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None)? };

		Ok(Shader {
			context: context,
			pipeline_layout: pipeline_layout,
		})
	}
}

impl Drop for Shader<'_> {
	fn drop(&mut self) {
		unsafe {
			self.context.device.destroy_pipeline_layout(self.pipeline_layout, None);
		}
	}
}
