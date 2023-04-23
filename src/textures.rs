#[path = "utils.rs"] mod utils;
#[path = "aqua/png.rs"] mod png;


use std ::{
	error::Error,
};

use std::ptr::copy_nonoverlapping;

extern crate ash;
pub struct Texture {
    pub image:  ash::vk::Image,
    pub memory: ash::vk::DeviceMemory,
    pub view: ash::vk::ImageView,
    pub sampler: Option<ash::vk::Sampler>,
}



impl Texture {

   pub unsafe fn begin_single_time_commands(
        device: &ash::Device,
        command_pool : ash::vk::CommandPool,
    ) -> Result<ash::vk::CommandBuffer, Box<dyn Error>> 
    {
        let info = ash::vk::CommandBufferAllocateInfo::default()
            .level(ash::vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);
    
        let command_buffer = device.allocate_command_buffers(&info)?[0];
    
        let info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    
        device.begin_command_buffer(command_buffer, &info)?;
    
        Ok(command_buffer)
    }

    pub unsafe fn end_single_time_commands(
        device: &ash::Device,
        command_buffer: ash::vk::CommandBuffer,
        graphic_queue : ash::vk::Queue,
        command_pool : ash::vk::CommandPool,
    ) ->  Result<(), Box<dyn Error>>  {
        device.end_command_buffer(command_buffer)?;
    
        let command_buffers = &[command_buffer];
        let info = ash::vk::SubmitInfo::default()
            .command_buffers(command_buffers);
    
        device.queue_submit(graphic_queue, &[info], ash::vk::Fence::null())?;
        device.queue_wait_idle(graphic_queue)?;
    
        device.free_command_buffers(command_pool, &[command_buffer]);
    
        Ok(())
    }

    unsafe fn copy_buffer_to_image(
        device: &ash::Device,
        buffer: ash::vk::Buffer,
        image: ash::vk::Image,
        width: u32,
        height: u32,
        command_pool : ash::vk::CommandPool,
        graphic_queue : ash::vk::Queue,
    ) -> Result<(), Box<dyn Error>> {
        let command_buffer = Self::begin_single_time_commands(device, command_pool)?;
    
        let subresource = ash::vk::ImageSubresourceLayers::default()
            .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
    
        let region = ash::vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(ash::vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(ash::vk::Extent3D {
                width,
                height,
                depth: 1,
            });
    
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    
        Self::end_single_time_commands(device, command_buffer, graphic_queue, command_pool)?;
    
        Ok(())
    }

    unsafe fn transition_image_layout(
        device: &ash::Device,
        image: ash::vk::Image,
        format: ash::vk::Format,
        old_layout: ash::vk::ImageLayout,
        new_layout: ash::vk::ImageLayout,
        command_pool : ash::vk::CommandPool,
        graphic_queue : ash::vk::Queue,
    ) -> Result<(), Box<dyn Error>> {
        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) = match (old_layout, new_layout) {
            (ash::vk::ImageLayout::UNDEFINED, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                ash::vk::AccessFlags::empty(),
                ash::vk::AccessFlags::TRANSFER_WRITE,
                ash::vk::PipelineStageFlags::TOP_OF_PIPE,
                ash::vk::PipelineStageFlags::TRANSFER,
            ),
            (ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                ash::vk::AccessFlags::TRANSFER_WRITE,
                ash::vk::AccessFlags::SHADER_READ,
                ash::vk::PipelineStageFlags::TRANSFER,
                ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => return Ok(()),
        };
    
        let command_buffer = Self::begin_single_time_commands(device, command_pool)?;
    
        let subresource = ash::vk::ImageSubresourceRange::default()
            .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
    
        let barrier = ash::vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);
    
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            ash::vk::DependencyFlags::empty(),
            &[] as &[ash::vk::MemoryBarrier],
            &[] as &[ash::vk::BufferMemoryBarrier],
            &[barrier],
        );
    
        Self::end_single_time_commands(device, command_buffer, graphic_queue, command_pool)?;
    
        Ok(())
    }


    pub fn create_image_from_path(graphic_queue : ash::vk::Queue , device : &ash::Device,  memory_properties : ash::vk::PhysicalDeviceMemoryProperties, command_pool : ash::vk::CommandPool, path : String){
        println!("sssssssssssssssssss");
        let mut png = png::Png::from_path(&path);
        println!("pdosfo^ids^dso^dsdsf");
        let mut png_result = png.draw();
        
        let image_extent = ash::vk::Extent2D { width : png_result.width , height : png_result.height };
        
        // CREATE STAGGING BUFFER

        let buffer_info = ash::vk::BufferCreateInfo::default()
        .size((png_result.bpp/8 * image_extent.width * image_extent.height) as u64)
        .usage(ash::vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let staging_buffer = unsafe { device.create_buffer(&buffer_info, None).unwrap() };

        // Memory

        let requirements = unsafe { device.get_buffer_memory_requirements(staging_buffer) };

        let memory_info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(utils::find_memory_type(requirements, memory_properties, ash::vk::MemoryPropertyFlags::HOST_COHERENT | ash::vk::MemoryPropertyFlags::HOST_VISIBLE));

        let staging_buffer_memory = unsafe { device.allocate_memory(&memory_info, None).unwrap() } ;

        unsafe { device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0).unwrap() };

        // Copy (staging)

        let memory = unsafe { device.map_memory(staging_buffer_memory, 0, (png_result.bpp/8 * image_extent.width * image_extent.height) as u64, ash::vk::MemoryMapFlags::empty()).unwrap() };

        unsafe { copy_nonoverlapping(png_result.buf.as_ptr(), memory.cast(), (png_result.bpp/8 * image_extent.width * image_extent.height) as usize) };

        unsafe { device.unmap_memory(staging_buffer_memory) };

        //Create Image

        let info = ash::vk::ImageCreateInfo::default()
        .image_type(ash::vk::ImageType::TYPE_2D)
        .extent(ash::vk::Extent3D {
            width : image_extent.width,
            height : image_extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(ash::vk::Format::R8G8B8A8_SRGB)
        .tiling(ash::vk::ImageTiling::OPTIMAL)
        .initial_layout(ash::vk::ImageLayout::UNDEFINED)
        .usage( ash::vk::ImageUsageFlags::SAMPLED | ash::vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
        .samples(ash::vk::SampleCountFlags::TYPE_1);

        let texture_image = unsafe { device.create_image(&info, None).unwrap() };

        // Memory

        let requirements = unsafe { device.get_image_memory_requirements(texture_image) };

        let info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(utils::find_memory_type(requirements, memory_properties, ash::vk::MemoryPropertyFlags::HOST_COHERENT | ash::vk::MemoryPropertyFlags::HOST_VISIBLE));

        let texture_image_memory = unsafe { device.allocate_memory(&info, None).unwrap() };

        unsafe { device.bind_image_memory(texture_image, texture_image_memory, 0).unwrap() };

        unsafe{
            Self::transition_image_layout(
                device,
                texture_image,
                ash::vk::Format::R8G8B8A8_SRGB,
                ash::vk::ImageLayout::UNDEFINED,
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                command_pool,
                graphic_queue
            ).unwrap();
        }
        //TODO FAIRE LE TRUC DU MEC AY DESSUS
        unsafe { Self::copy_buffer_to_image(device, staging_buffer, texture_image, image_extent.width, image_extent.height, command_pool, graphic_queue).unwrap() };


        unsafe{
            Self::transition_image_layout(
                device,
                texture_image,
                ash::vk::Format::R8G8B8A8_SRGB,
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                command_pool,
                graphic_queue
            ).unwrap();
        }
    
        unsafe 
        {   
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        //* CE QU'ON RECUÉPÈRE:  data.texture_image */
    }

    fn create_image_view( image : ash::vk::Image, format : ash::vk::Format, aspectFlags : ash::vk::ImageAspectFlags) 
    {

    }

    pub fn create_image( device : &ash::Device, memory_properties : ash::vk::PhysicalDeviceMemoryProperties, memory_properties_flag : ash::vk::MemoryPropertyFlags, 
    width : u32, height :u32, format : ash::vk::Format, tiling : ash::vk::ImageTiling
    , image_type : ash::vk::ImageType, mip_levels : u32, usage : ash::vk::ImageUsageFlags, sample_count_flag : ash::vk::SampleCountFlags) -> (ash::vk::Image, ash::vk::DeviceMemory) 

    {
        //TODO Voir si on doit setup des flags ou mettre en paramètre

        let image_info = ash::vk::ImageCreateInfo::default()
            .image_type(image_type)
            .extent(ash::vk::Extent3D{
                width : width,
                height : height,
                depth : 1,  
            })
            .mip_levels(mip_levels)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .samples(sample_count_flag)
            .flags(ash::vk::ImageCreateFlags::empty());
        
        let image = unsafe { device.create_image(&image_info, None).unwrap()} ;
        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        
         let memory_type_index = utils::find_memory_type(memory_requirements, memory_properties, memory_properties_flag);
        
        let allocate_info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);
        
        let memory = unsafe 
        { 
            let mem = device.allocate_memory(&allocate_info, None).unwrap();
            device.bind_image_memory(image, mem, 0).unwrap();
            mem 
        };
        (image, memory)
    }

}