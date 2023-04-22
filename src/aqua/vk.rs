extern crate ash;

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
	pub instance: ash::Instance,
}

impl VkContext {
	pub fn new(win: aqua::win::Win, name: &str, ver_major: u32, ver_minor: u32, ver_patch: u32) -> VkContext {
		let kind = VkContextKind::Win;
		let c_str = std::ffi::CString::new(name).unwrap();

		let dev = aqua::query_device("aquabsd.alps.vk");
		let context = aqua::send_device!(dev, 0x6363, kind.to_device_arg(), win.win, c_str.as_ptr(), ver_major, ver_minor, ver_patch);

		// TODO check for failures

		let fp_addr = Self::get_fn(dev, context, "vkGetInstanceProcAddr");
		let fp: ash::vk::PFN_vkGetInstanceProcAddr = unsafe { std::mem::transmute(fp_addr) };

		let static_fn = ash::vk::StaticFn {
			get_instance_proc_addr: fp,
		};

		let handle = unsafe { Self::get_instance(dev, context) };
		let instance = unsafe { ash::Instance::load(&static_fn, handle) };

		VkContext {
			dev: dev,
			context: context,
			instance: instance,
		}
	}

	fn get_fn(dev: aqua::Device, context: u64, name: &str) -> u64 {
		let c_str = std::ffi::CString::new(name).unwrap();
		aqua::send_device!(dev, 0x6766, context, c_str.as_ptr())
	}

	unsafe fn get_instance(dev: aqua::Device, context: u64) -> ash::vk::Instance {
		let instance_addr = aqua::send_device!(dev, 0x6769, context);
		std::mem::transmute(instance_addr)
	}
}

impl Drop for VkContext {
	fn drop(&mut self) {
		aqua::send_device!(self.dev, 0x6364, self.context);
	}
}
