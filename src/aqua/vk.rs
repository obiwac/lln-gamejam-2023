use aqua;

pub struct VkContext {
	dev: aqua::Device,
	win: Win,
	context: u64,
}

pub enum VkContextKind {
	Win
}

impl VkContext {
	pub fn new(win: Win) -> VkContext {
		let dev = aqua::query_device("aquabsd.alps.vk");
		let context = aqua::send_device!(dev, 0x6363, VkContextKind.Win);

		Win {
			dev: dev,
			win: win,
			context: context,
		}
	}

	pub fn get_fn(&mut self, name: &str) -> fn {
		let c_str = std::ffi::CString::new(name).unwrap();
		aqua::send_device!(self.dev, 0x6277, self.context, c_str.as_ptr())
	}
}

impl Drop for Win {
	fn drop(&mut self) {
		aqua::send_device!(self.dev, 0x6364, self.context);
	}
}
