extern crate ash;

use aqua;

pub enum VkContextKind {
	Win
}

impl VkContextKind {
	fn to_device_arg(&self) -> u64 {
		match self {
			Self::Win => 0,
		}
	}
}

pub struct VkContext {
	dev: aqua::Device,
	context: u64,
	entry: ash::Entry,
	fp: ash::vk::PFN_vkGetInstanceProcAddr,
}

impl VkContext {
	pub fn new(win: &aqua::win::Win, name: &str, ver_major: u32, ver_minor: u32, ver_patch: u32) -> VkContext {
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

		let entry = unsafe { ash::Entry::from_static_fn(static_fn) };

		VkContext {
			dev: dev,
			context: context,
			entry: entry,
			fp: fp,
		}
	}

	fn get_fn(dev: aqua::Device, context: u64, name: &str) -> u64 {
		let c_str = std::ffi::CString::new(name).unwrap();
		aqua::send_device!(dev, 0x6766, context, c_str.as_ptr())
	}

	unsafe fn get_vk_instance(dev: aqua::Device, context: u64) -> ash::vk::Instance {
		let instance_addr = aqua::send_device!(dev, 0x6769, context);
		std::mem::transmute(instance_addr)
	}

	pub fn get_instance(&mut self) -> ash::Instance {
		let static_fn = ash::vk::StaticFn {
			get_instance_proc_addr: self.fp,
		};

		let instance = unsafe { Self::get_vk_instance(self.dev, self.context) };
		unsafe { ash::Instance::load(&static_fn, instance) }
	}

	unsafe fn get_vk_device(dev: aqua::Device, context: u64) -> ash::vk::Device {
		let addr = aqua::send_device!(dev, 0x6763, context);
		std::mem::transmute(addr)
	}

	pub fn get_device(&mut self) -> ash::Device {
		let instance = self.get_instance(); // XXX meh, cache this (can I get this from entry instead?)
		let device = unsafe { Self::get_vk_device(self.dev, self.context) };

		unsafe { ash::Device::load(&instance.instance_fn_1_0, device) }
	}

	pub fn get_phys_device(&mut self) -> ash::vk::PhysicalDevice {
		let addr = aqua::send_device!(self.dev, 0x6770, self.context);
		unsafe { *std::mem::transmute::<u64, *const ash::vk::PhysicalDevice>(addr) }
	}

	pub fn get_surface(&mut self) -> ash::extensions::khr::Surface {
		let instance = self.get_instance(); // XXX meh, cache this (can I get this from entry instead?)
		ash::extensions::khr::Surface::new(&self.entry, &instance)
	}

	pub fn get_surface_khr(&mut self) -> ash::vk::SurfaceKHR {
		let addr = aqua::send_device!(self.dev, 0x6773, self.context);
		unsafe { *std::mem::transmute::<u64, *const ash::vk::SurfaceKHR>(addr) }
	}

	pub fn get_graphics_queue(&mut self) -> u32 {
		aqua::send_device!(self.dev, 0x6771, self.context) as u32
	}

	pub fn get_present_queue(&mut self) -> u32 {
		aqua::send_device!(self.dev, 0x7071, self.context) as u32
	}

}

impl Drop for VkContext {
	fn drop(&mut self) {
		aqua::send_device!(self.dev, 0x6364, self.context);
	}
}
