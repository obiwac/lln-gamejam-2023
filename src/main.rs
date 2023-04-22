mod aqua;
use std ::{
	error::Error,
};
extern crate ash;

fn main() -> Result<(), Box<dyn Error>> {
	let name = "Louvain-li-Nux Gamejam 2023";

	let mut win = aqua::win::Win::new(800, 600);
	win.caption(name);

	println!("get vk_context");
	let mut vk_context = aqua::vk::VkContext::new(win, name, 0, 1, 0);

	println!("get instance");
	let instance = &vk_context.get_instance();

	println!("get device");
	let device = &vk_context.get_device();

	println!("get physical device");
	let phys_device = *vk_context.get_phys_device();

	println!("get surface");
	let surface = &vk_context.get_surface();

	println!("get vk)surface");
	let vk_surface = vk_context.get_surface_khr();

	// Create the swapchain 

	println!("get format");

	let format = {
		let formats =
			unsafe { surface.get_physical_device_surface_formats(phys_device, vk_surface)? };
		if formats.len() == 1 && formats[0].format == ash::vk::Format::UNDEFINED {
			ash::vk::SurfaceFormatKHR {
				format: ash::vk::Format::B8G8R8A8_UNORM,
				color_space: ash::vk::ColorSpaceKHR::SRGB_NONLINEAR,
			}
		} else {
			*formats
				.iter()
				.find(|format| {
					format.format == ash::vk::Format::B8G8R8A8_UNORM
						&& format.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR
				})
				.unwrap_or(&formats[0])
		}
	};

	//println!("Swapchain format: {:?}", format);

	let swapchain = ash::extensions::khr::Swapchain::new(instance, device); 
	/*let swapchain_create_info =  ash::vk::SwapchainCreateInfoKHR::build()
	.surface(surface);	
	*/

	println!("Ceci est un teste 2");

	std::thread::sleep(std::time::Duration::from_millis(1000));

	Ok(())
}
