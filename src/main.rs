mod aqua;
use std ::{
	error::Error,
};
extern crate ash;


fn main() -> Result<(), Box<dyn Error>> {
	let name = "Louvain-li-Nux Gamejam 2023";
	
	const WIDTH : u32 = 800;
	const HEIGTH : u32 = 600;

	let mut win = aqua::win::Win::new(WIDTH, HEIGTH);
	win.caption(name);

	println!("get vk_context");
	let mut vk_context = aqua::vk::VkContext::new(win, name, 0, 1, 0);

	println!("get instance");
	let instance = &vk_context.get_instance();

	println!("get device");
	let device = &vk_context.get_device();

	println!("get physical device");
	let phys_device = vk_context.get_phys_device();

	println!("get surface");
	let surface = &vk_context.get_surface();

	println!("get vk)surface");
	let vk_surface = vk_context.get_surface_khr();

	// Create the swapchain 

	println!("get format {:?}", phys_device);

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

	let present_mode = {
        let present_modes = unsafe {
            surface
                .get_physical_device_surface_present_modes(phys_device, vk_surface)
                .expect("Failed to get physical device surface present modes")
        };
        if present_modes.contains(&ash::vk::PresentModeKHR::IMMEDIATE) {
            ash::vk::PresentModeKHR::IMMEDIATE
        } else {
            ash::vk::PresentModeKHR::FIFO
        }
    };

	let capabilities = unsafe { surface.get_physical_device_surface_capabilities(phys_device, vk_surface)? };
		
	let extent = {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = WIDTH.min(max.width).max(min.width);
            let height = HEIGTH.min(max.height).max(min.height);
            ash::vk::Extent2D { width, height }
        }
    }; 

	let image_count = capabilities.min_image_count;

	println!("Device capabilites {:?}", capabilities);
	println!("Swapchain format: {:?}", format);
	println!("Swapchain present mode : {:?} ", present_mode);
	println!("Swapchain extends : {:?} ", extent);
	println!("Swapchain immage count : {:?}", image_count);

	let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device); 
	let swapchain_create_info = ash::vk::SwapchainCreateInfoKHR::default()
		.surface(vk_surface)
		.min_image_count(image_count)
		.image_format(format.format)
		.image_color_space(format.color_space)
		.image_extent(extent)
		.image_array_layers(1)
		.image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
		.image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE) 
		.pre_transform(capabilities.current_transform)
		.composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE) 
		.present_mode(present_mode) 
		.clipped(true); // TODO je sais pas ce que ça fait

	println!("swpachain create info : {:?}", swapchain_create_info);

    let swapchain_khr = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };


	println!("Ceci est un teste 2");

	std::thread::sleep(std::time::Duration::from_millis(1000));

	unsafe{swapchain_loader.destroy_swapchain(swapchain_khr, None)};
	Ok(())
}
