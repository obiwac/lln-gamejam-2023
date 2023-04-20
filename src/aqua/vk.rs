use aqua;

pub enum VkContextKind {
	Win
}

impl VkContextKind {
	fn to_device_arg(&self) -> u64 {
		match self {
			VkContextKind::Win => 0,
		}
	}
}

pub struct VkContext {
	dev: aqua::Device,
	context: u64,
}

impl VkContext {
	pub fn new(win: aqua::win::Win, name: &str, ver_major: u32, ver_minor: u32, ver_patch: u32) -> VkContext {
		let kind = VkContextKind::Win;
		let c_str = std::ffi::CString::new(name).unwrap();

		let dev = aqua::query_device("aquabsd.alps.vk");
		let context = aqua::send_device!(dev, 0x6363, kind.to_device_arg(), c_str.as_ptr(), ver_major, ver_minor, ver_patch);

		// TODO check for failures

		VkContext {
			dev: dev,
			context: context,
		}
	}

	pub fn get_fn(&mut self, name: &str) -> u64 {
		let c_str = std::ffi::CString::new(name).unwrap();
		aqua::send_device!(self.dev, 0x6277, self.context, c_str.as_ptr())
	}
}

impl Drop for VkContext {
	fn drop(&mut self) {
		aqua::send_device!(self.dev, 0x6364, self.context);
	}
}
