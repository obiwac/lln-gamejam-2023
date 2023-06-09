extern crate aqua;

mod buffers;
mod player;
mod textures;
mod utils;
use std::error::Error;

mod shader;

extern crate ash;

pub struct Context<'a> {
	image_available_semaphore: ash::vk::Semaphore,
	render_finished_semaphore: ash::vk::Semaphore,
	in_flight_fence: ash::vk::Fence,

	swapchain_loader: &'a ash::extensions::khr::Swapchain,
	swapchain_khr: ash::vk::SwapchainKHR,

	device: &'a ash::Device,

	command_buffers: std::vec::Vec<ash::vk::CommandBuffer>,
	framebuffers: &'a std::vec::Vec<ash::vk::Framebuffer>,

	render_pass_khr: ash::vk::RenderPass,
	extent: ash::vk::Extent2D,
	q_family: ash::vk::Queue,
	q_present: ash::vk::Queue,

	ibo: buffers::Indexbuffer,
	shader: shader::Shader<'a>,
	player: player::Player,
	descriptor_sets: Vec<ash::vk::DescriptorSet>,
}

static mut test: f32 = -1.0;
static mut target_test: f32 = -1.0;

fn draw(ctx: &mut Context) -> Result<(), Box<dyn Error>> {

	let mut mouse = aqua::mouse::Mouse::default();
	mouse.update();

	
	let x = mouse.poll_axis(aqua::mouse::MouseAxis::X);
	let y = mouse.poll_axis(aqua::mouse::MouseAxis::Y);
	let z = mouse.poll_axis(aqua::mouse::MouseAxis::Z) / 10.;
	
	unsafe {
		ctx.player.p_mat.identity();
		ctx.player
			.p_mat
			.perspective(93.0, 800.0 / 600.0, 0.001, 50.0);

		ctx.player.mv_mat.identity();
		ctx.player.mv_mat.translate(0.0, 0.0, test);
		ctx.player.mv_mat = ctx
			.player
			.mv_mat
			.rotate_2d(x * 6.28,-y * 6.28 + 6.28 / 2.0);

		target_test += z;
		test += (target_test - test) / 100.0;

		if (target_test > 0.01) {
			target_test = 0.01;
		}
	}

	let mvp_mat = ctx.player.p_mat.mul(&ctx.player.mv_mat);

	// cursed Vulkan shit

	unsafe {
		ctx.device
			.wait_for_fences(&[ctx.in_flight_fence], true, std::u64::MAX)?
	};
	unsafe { ctx.device.reset_fences(&[ctx.in_flight_fence])? };

	let next_image_frame = unsafe {
		ctx.swapchain_loader.acquire_next_image(
			ctx.swapchain_khr,
			std::u64::MAX,
			ctx.image_available_semaphore,
			ash::vk::Fence::null(),
		)
	};

	let image_index = match next_image_frame {
		Ok((image_index, _)) => image_index,
		Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
			return Ok(());
		}
		Err(error) => panic!("Error while acquiring next image. Cause: {}", error),
	};
	//* HOPE next_image_frame has not failed ...

	// Reset the command buffer ....
	// Start a new record wouhouuuu

	let current_command_buffer = ctx.command_buffers[image_index as usize];

	unsafe {
		let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default()
			.flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

		ctx.device
			.begin_command_buffer(current_command_buffer, &command_buffer_begin_info)?;
		let raw_mat: [u8; 64] = std::mem::transmute(mvp_mat.mat);

		ctx.device.cmd_push_constants(
			current_command_buffer,
			ctx.shader.pipeline_layout,
			ash::vk::ShaderStageFlags::VERTEX,
			0,
			&raw_mat,
		);
		ctx.device.cmd_bind_descriptor_sets(
			current_command_buffer,
			ash::vk::PipelineBindPoint::GRAPHICS,
			ctx.shader.pipeline_layout,
			0,
			&ctx.descriptor_sets[..],
			&[],
		);

		let render_pass_begin_info = ash::vk::RenderPassBeginInfo::default()
			.render_pass(ctx.render_pass_khr)
			.framebuffer(ctx.framebuffers[image_index as usize])
			.render_area(ash::vk::Rect2D {
				offset: ash::vk::Offset2D { x: 0, y: 0 },
				extent: ctx.extent,
			})
			.clear_values(&[ash::vk::ClearValue {
				color: ash::vk::ClearColorValue {
					float32: [0.0f32, 0.0f32, 1.0f32, 1.0f32],
				},
			}]);

		ctx.device.cmd_begin_render_pass(
			current_command_buffer,
			&render_pass_begin_info,
			ash::vk::SubpassContents::INLINE,
		);
		ctx.device.cmd_bind_pipeline(
			current_command_buffer,
			ash::vk::PipelineBindPoint::GRAPHICS,
			ctx.shader.pipeline,
		);

		ctx.device
			.cmd_set_viewport(current_command_buffer, 0, &ctx.shader.viewports);
		ctx.device
			.cmd_set_scissor(current_command_buffer, 0, &ctx.shader.scissors);

		ctx.device.cmd_bind_index_buffer(
			current_command_buffer,
			ctx.ibo.ibo,
			0,
			ash::vk::IndexType::UINT32,
		);
		ctx.device
			.cmd_draw_indexed(current_command_buffer, 6, 1, 0, 0, 1);

		// ctx.device.cmd_draw(current_command_buffer, 3, 1, 0, 0);

		ctx.device.cmd_end_render_pass(current_command_buffer);
		ctx.device.end_command_buffer(current_command_buffer)?;
	}

	let a_available_semaphore = [ctx.image_available_semaphore];
	let a_current_command_buffer = [current_command_buffer];
	let a_render_finished_semaphore = [ctx.render_finished_semaphore];
	let a_swapchain_khr = [ctx.swapchain_khr];
	let a_image_index = [image_index];
	// Submit the command buffer
	let submit_info = [ash::vk::SubmitInfo::default()
		.wait_semaphores(&a_available_semaphore)
		.wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
		.command_buffers(&a_current_command_buffer)
		.signal_semaphores(&a_render_finished_semaphore)];

	unsafe {
		ctx.device
			.queue_submit(ctx.q_family, &submit_info, ctx.in_flight_fence)
	}; // TODO HMMM

	// AND NOW PRESENT
	let present_info = ash::vk::PresentInfoKHR::default()
		.wait_semaphores(&a_render_finished_semaphore)
		.swapchains(&a_swapchain_khr)
		.image_indices(&a_image_index);

	let present_result = unsafe {
		ctx.swapchain_loader
			.queue_present(ctx.q_present, &present_info)
	};

	Ok(())
}

extern "C" fn draw_wrapper(win: u64, data: u64) -> u64 {
	let mut ctx: &mut Context = unsafe { std::mem::transmute(data) };

	/**********************************************************************/
	draw(ctx);
	0
}

#[allow(clippy::too_many_arguments)]
pub fn record_submit_commandbuffer<F: FnOnce(&ash::Device, ash::vk::CommandBuffer)>(
	device: &ash::Device,
	command_buffer: ash::vk::CommandBuffer,
	command_buffer_reuse_fence: ash::vk::Fence,
	submit_queue: ash::vk::Queue,
	wait_mask: &[ash::vk::PipelineStageFlags],
	wait_semaphores: &[ash::vk::Semaphore],
	signal_semaphores: &[ash::vk::Semaphore],
	f: F,
) {
	unsafe {
		device
			.wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)
			.expect("Wait for fence failed.");

		device
			.reset_fences(&[command_buffer_reuse_fence])
			.expect("Reset fences failed.");

		device
			.reset_command_buffer(
				command_buffer,
				ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
			)
			.expect("Reset command buffer failed.");

		let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default()
			.flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		device
			.begin_command_buffer(command_buffer, &command_buffer_begin_info)
			.expect("Begin commandbuffer");
		f(device, command_buffer);
		device
			.end_command_buffer(command_buffer)
			.expect("End commandbuffer");

		let command_buffers = vec![command_buffer];

		let submit_info = ash::vk::SubmitInfo::default()
			.wait_semaphores(wait_semaphores)
			.wait_dst_stage_mask(wait_mask)
			.command_buffers(&command_buffers)
			.signal_semaphores(signal_semaphores);

		device
			.queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
			.expect("queue submit failed.");
	}
}

#[no_mangle]
pub fn main() -> Result<(), Box<dyn Error>> {
	let name = "Louvain-li-Nux Gamejam 2023";

	const WIDTH: u32 = 800;
	const HEIGHT: u32 = 600;

	let mut win = aqua::win::Win::new(WIDTH, HEIGHT);
	win.caption(name);

	let mut vk_context = aqua::vk::VkContext::new(&win, name, 0, 1, 0);
	let instance = &vk_context.get_instance();
	let device = &vk_context.get_device();
	let phys_device = vk_context.get_phys_device();
	let surface = &vk_context.get_surface();
	let surface_khr = vk_context.get_surface_khr();

	let q_family_index = vk_context.get_graphics_queue();
	let q_present_index = vk_context.get_present_queue();

	let q_family = unsafe { device.get_device_queue(q_family_index, 0) };
	let q_present = unsafe { device.get_device_queue(q_present_index, 0) };

	let memory_properties = unsafe { instance.get_physical_device_memory_properties(phys_device) };
	println!("Memory properties {:?}", memory_properties);
	// Create the swapchain

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

	let capabilities =
		unsafe { surface.get_physical_device_surface_capabilities(phys_device, surface_khr)? };

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
	let images = unsafe { swapchain_loader.get_swapchain_images(swapchain_khr)? };
	let images_view = images
		.iter()
		.map(|image| {
			let create_info = ash::vk::ImageViewCreateInfo::default()
				.image(*image)
				.view_type(ash::vk::ImageViewType::TYPE_2D)
				.format(format.format)
				.subresource_range(ash::vk::ImageSubresourceRange {
					aspect_mask: ash::vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				});

			unsafe { device.create_image_view(&create_info, None) }
		})
		.collect::<Result<std::vec::Vec<_>, _>>()?;

	println!("On a crée la swapchain");

	//Command Pool
	// and get grapgics family

	let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default()
		.queue_family_index(q_family_index)
		.flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); //TODO Check flag...

	let command_pool_khr = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

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
			ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
				| ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
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

	let render_pass_khr = unsafe { device.create_render_pass(&render_pass_info, None)? };
	println!("Created render pass");

	// Create Frame buffer now
	let framebuffers = images_view
		.iter()
		.map(|view| [*view])
		.map(|attachments| {
			let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
				.render_pass(render_pass_khr)
				.attachments(&attachments)
				.width(extent.width)
				.height(extent.height)
				.layers(1);
			println!("Create Info :  {:?}", frame_buffer_create_info);
			unsafe { device.create_framebuffer(&frame_buffer_create_info, None) }
		})
		.collect::<Result<std::vec::Vec<_>, _>>()?;

	// Command buffer
	let command_buffers = {
		let allocation_info = ash::vk::CommandBufferAllocateInfo::default()
			.command_pool(command_pool_khr)
			.level(ash::vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(images.len() as _);
		unsafe { device.allocate_command_buffers(&allocation_info)? }
	};

	let setup_command_buffer = command_buffers[0];
	let draw_command_buffer = command_buffers[1];

	// Start the rendering ......;
	// All clean and submit all model to the current buffer ....
	println!(
		"Number of frame buffer & command buffer : {:?}",
		command_buffers.iter().len()
	);

	let image_available_semaphore = {
		let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();
		//TODO :	 semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
		unsafe { device.create_semaphore(&semaphore_create_info, None)? }
	};

	let render_finished_semaphore = {
		let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();
		unsafe { device.create_semaphore(&semaphore_create_info, None)? }
	};

	let in_flight_fence = {
		let fence_info =
			ash::vk::FenceCreateInfo::default().flags(ash::vk::FenceCreateFlags::SIGNALED); // Tricks to not wait for the first render
		unsafe { device.create_fence(&fence_info, None)? }
	};

	let index_buffer_data = [0u32, 1, 2];
	let ibo = buffers::Indexbuffer::new(device, memory_properties, index_buffer_data.to_vec());

	// Create depth ressources :
	/*textures::Texture::create_image(device, memory_properties, ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
		extent.width, extent.height, ash::vk::Format::D32_SFLOAT, ash::vk::ImageTiling::OPTIMAL, ash::vk::ImageType::TYPE_2D
	,1 , ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,  ash::vk::SampleCountFlags::TYPE_1);
	*/
	println!("ojides^jio^idfsodfsoojidfs");
	let pig_texture = textures::Texture::create_image_from_path(
		q_family,
		device,
		memory_properties,
		command_pool_khr,
		"res/obama.png".to_string(),
	)
	.unwrap();

	let sampler_info = ash::vk::SamplerCreateInfo {
		mag_filter: ash::vk::Filter::LINEAR,
		min_filter: ash::vk::Filter::LINEAR,
		mipmap_mode: ash::vk::SamplerMipmapMode::LINEAR,
		address_mode_u: ash::vk::SamplerAddressMode::MIRRORED_REPEAT,
		address_mode_v: ash::vk::SamplerAddressMode::MIRRORED_REPEAT,
		address_mode_w: ash::vk::SamplerAddressMode::MIRRORED_REPEAT,
		max_anisotropy: 1.0,
		border_color: ash::vk::BorderColor::FLOAT_OPAQUE_WHITE,
		compare_op: ash::vk::CompareOp::NEVER,
		..Default::default()
	};

	let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

	let descriptor_pool_sizes = [
		ash::vk::DescriptorPoolSize::default()
			.ty(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(1) // MAX_FRAMES_IN_FLIGHT
	];

	let descriptor_pool_create_info = ash::vk::DescriptorPoolCreateInfo::default()
		.pool_sizes(&descriptor_pool_sizes)
		.max_sets(1);









	let layout_sampler_bindings = [
		ash::vk::DescriptorSetLayoutBinding::default()
			.binding(1u32)
			.descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(1u32)
			.stage_flags(ash::vk::ShaderStageFlags::FRAGMENT)
	];

	let layout_info = ash::vk::DescriptorSetLayoutCreateInfo::default()
		.bindings(&layout_sampler_bindings);

	let descriptor_layouts = [unsafe {
		device
			.create_descriptor_set_layout(&layout_info, None.as_ref())
			.unwrap()
	}];

	let descriptor_pool = unsafe {
		device.create_descriptor_pool(&descriptor_pool_create_info, None.as_ref()).unwrap()
	};

	let descriptor_set_info = ash::vk::DescriptorSetAllocateInfo::default()
		.descriptor_pool(descriptor_pool)
		.set_layouts(&descriptor_layouts);

	let descriptor_sets = unsafe { device.allocate_descriptor_sets(&descriptor_set_info) };

	let descriptor_image_info = ash::vk::DescriptorImageInfo::default()
		.sampler(sampler)
		.image_view(pig_texture.view)
		.image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

	let descriptor_set = descriptor_sets.clone()?[0];

	let mut descriptor_write = ash::vk::WriteDescriptorSet {
		dst_set: descriptor_set,
		dst_binding: 1,
		descriptor_count: 1,
		descriptor_type: ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
		p_image_info: &descriptor_image_info,
		..Default::default()
	};

	unsafe { device.update_descriptor_sets(&[descriptor_write], &[]) };

	let shader = shader::Shader::new(
		&device,
		extent,
		render_pass_khr,
		&descriptor_layouts[..],
		"src/shaders/shader.vert.spv",
		"src/shaders/shader.frag.spv",
	)?;

	let mut player = player::Player::new();

	let context = Context {
		image_available_semaphore: image_available_semaphore,
		command_buffers: command_buffers,
		device: device,
		extent: extent,
		framebuffers: &framebuffers,
		in_flight_fence: in_flight_fence,
		q_family: q_family,
		q_present: q_present,
		render_finished_semaphore: render_finished_semaphore,
		render_pass_khr: render_pass_khr,
		swapchain_khr: swapchain_khr,
		swapchain_loader: &swapchain_loader,
		ibo: ibo.unwrap(),
		shader: shader,
		player: player,
		descriptor_sets: descriptor_sets?,
	};

	// draw loop

	win.draw_hook(draw_wrapper, unsafe { std::mem::transmute(&context) });
	win.draw_loop();

	// Destroy things

	unsafe { swapchain_loader.destroy_swapchain(swapchain_khr, None) };
	unsafe { device.destroy_command_pool(command_pool_khr, None) };
	unsafe { device.destroy_render_pass(render_pass_khr, None) };
	unsafe {
		images_view
			.iter()
			.for_each(|v| device.destroy_image_view(*v, None))
	};
	unsafe {
		framebuffers
			.iter()
			.for_each(|f| device.destroy_framebuffer(*f, None))
	};
	unsafe { device.destroy_semaphore(image_available_semaphore, None) };
	unsafe { device.destroy_semaphore(render_finished_semaphore, None) };
	unsafe { device.destroy_fence(in_flight_fence, None) };
	unsafe { device.destroy_sampler(sampler, None) };
	Ok(())
}
