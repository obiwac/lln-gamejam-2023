mod aqua;
use std ::{
	error::Error,
};
extern crate ash;

extern crate ndarray;

extern "C" fn draw(win: u64, data: u64) -> u64 {
	println!("Draw hook {:} {:}", win, data);
	0
}

fn main() -> Result<(), Box<dyn Error>> {
	let name = "Louvain-li-Nux Gamejam 2023";
	
	const WIDTH: u32 = 800;
	const HEIGHT: u32 = 600;

	let mut win = aqua::win::Win::new(WIDTH, HEIGHT);
	win.caption(name);

	println!("get vk_context");
	let mut vk_context = aqua::vk::VkContext::new(&win, name, 0, 1, 0);

	println!("get instance");
	let instance = &vk_context.get_instance();

	println!("get device");
	let device = &vk_context.get_device();

	println!("get physical device");
	let phys_device = vk_context.get_phys_device();

	println!("get surface");
	let surface = &vk_context.get_surface();

	println!("get vk)surface");
	let surface_khr = vk_context.get_surface_khr();

	let q_family = vk_context.get_queue();
	// Create the swapchain 

	println!("get format {:?}", surface_khr);
	println!("LA FAMILLE : {:?}",q_family);

	let format = {
		let formats =
			unsafe { surface.get_physical_device_surface_formats(phys_device, surface_khr)? };
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
                .get_physical_device_surface_present_modes(phys_device, surface_khr)
                .expect("Failed to get physical device surface present modes")
        };
        if present_modes.contains(&ash::vk::PresentModeKHR::IMMEDIATE) {
            ash::vk::PresentModeKHR::IMMEDIATE
        } else {
            ash::vk::PresentModeKHR::FIFO
        }
    };

	let capabilities = unsafe { surface.get_physical_device_surface_capabilities(phys_device, surface_khr)? };
		
	let extent = {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = WIDTH.min(max.width).max(min.width);
            let height = HEIGHT.min(max.height).max(min.height);
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
		.surface(surface_khr)
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

	// Create Image & Image views
	let images = unsafe { swapchain_loader.get_swapchain_images(swapchain_khr)?};
	let images_view = images
		.iter()
		.map(|image| 
		{
			let create_info = ash::vk::ImageViewCreateInfo::default()
				.image(*image)
				.view_type(ash::vk::ImageViewType::TYPE_2D)
                .format(format.format)
				.subresource_range(ash::vk::ImageSubresourceRange
				{
					aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
				});
			
			unsafe { device.create_image_view(&create_info, None) }
		}).collect::<Result<std::vec::Vec<_>, _>>()?;
		

	println!("On a crée la swapchain");

	//Command Pool
	// and get grapgics family

	let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default()
	.queue_family_index(q_family)
	.flags(ash::vk::CommandPoolCreateFlags::empty()); //TODO Check flag...	

	let command_pool_khr =  unsafe { device.create_command_pool(&command_pool_create_info, None)? };

	println!("Create command pool");

	// Create render pass
	let attachement_descritor = [ash::vk::AttachmentDescription::default()
		.format(format.format)
		.samples(ash::vk::SampleCountFlags::TYPE_1)
		.load_op(ash::vk::AttachmentLoadOp::CLEAR)
		.store_op(ash::vk::AttachmentStoreOp::STORE)
		.stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
		.stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(ash::vk::ImageLayout::UNDEFINED)
        .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)];

	let attachment_reference = [ash::vk::AttachmentReference::default()
		.attachment(0)
		.layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

	let subpass_deps = [ash::vk::SubpassDependency::default()
		.src_subpass(ash::vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(ash::vk::AccessFlags::empty())
        .dst_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            ash::vk::AccessFlags::COLOR_ATTACHMENT_READ | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )];


	let sub_pass_descriptor = [ash::vk::SubpassDescription::default()
		.pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
		//.color_attachment_count = 1
		.color_attachments(&attachment_reference)];

	let render_pass_info = ash::vk::RenderPassCreateInfo::default()
        .attachments(&attachement_descritor)
        .subpasses(&sub_pass_descriptor)
        .dependencies(&subpass_deps);
	
	println!("Render pass info  {:?}", render_pass_info);

	let render_pass_khr = unsafe {device.create_render_pass(&render_pass_info, None) ?};
	println!("Created render pass");

	// Create Frame buffer now
	let framebuffers = images_view.iter()
	.map(|view| [*view])
	.map(|attachments|
	{
		let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
			.render_pass(render_pass_khr)
			.attachments(&attachments)
			.width(extent.width)
			.height(extent.height)
			.layers(1);
		println!("Create Info :  {:?}", frame_buffer_create_info);
		unsafe  {device.create_framebuffer(&frame_buffer_create_info, None)}

	}).collect::<Result<std::vec::Vec<_>, _>>()?;


	// OK NOW THE PIPELINE !!!!
	//TODO FOR SHADER !!!!
	let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default();
	let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) ?};


	// Command buffer
	let command_buffers = 
	{
		let allocation_info = ash::vk::CommandBufferAllocateInfo::default() 
			.command_pool(command_pool_khr)
			.level(ash::vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(images.len() as _);
		unsafe { device.allocate_command_buffers(&allocation_info) ?}
	};

	// Start the rendering ......;
	// All clean and submit all model to the current buffer ....
	println!("Number of frame buffer & command buffer : {:?}", command_buffers.iter().len());
	/*
	for (index, buffer) in command_buffers.iter().enumerate()
	{
		let buff = *buffer;

		let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default()
			.flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
		unsafe { device.begin_command_buffer(buffer, &command_buffer_begin_info) ?}

		let render_pass_begin_info = ash::vk::RenderPassBeginInfo::default()
			.render_pass(render_pass_khr)
			.framebuffer(framebuffers[index])
			.render_area(ash::vk::Rect2D{
				offset : ash::vk::Offset2D {x : 0, y : 0},
				extent,
			})
			.clear_values(&[ash::vk::ClearValue {
				color : ash::vk::ClearColorValue{
					float32 : [0.5f32, 0.5f32, 0.5f32, 1.0f32], // Clear values
				},
			}]);

		// Here begin the affaires

	}*/


	std::thread::sleep(	std::time::Duration::from_millis(1000));

	win.draw_hook(draw, 1337);
	win.draw_loop();

	// Destroy things
	unsafe { swapchain_loader.destroy_swapchain(swapchain_khr, None) };
	unsafe { device.destroy_command_pool(command_pool_khr, None) };
	unsafe { device.destroy_render_pass(render_pass_khr, None) };
	unsafe { images_view.iter().for_each(|v| device.destroy_image_view(*v, None)) };
	unsafe { framebuffers.iter().for_each(|f| device.destroy_framebuffer(*f, None)) };
	unsafe { device.destroy_pipeline_layout(pipeline_layout, None) };
	Ok(())
}
