#[path = "utils.rs"] mod utils;
#[path = "aqua/png.rs"] mod png;


use ash::util::*;
use std::default::Default;
extern crate ash;
pub struct Texture {
    pub image:  ash::vk::Image,
    pub memory: ash::vk::DeviceMemory,
    pub view: ash::vk::ImageView,
    pub sampler: Option<ash::vk::Sampler>,
}



impl Texture {
    /* 
    fn begin_signle_command(command_pool : ash::vk::CommandPool)
    {
        let command_buffers = 
        {
            let allocation_info = ash::vk::CommandBufferAllocateInfo::default() 
                .command_pool(command_pool)
                .level(ash::vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(images.len() as _);
            unsafe { device.allocate_command_buffers(&allocation_info) ?}
        };
    }*/


    fn transition_image_layout( image : ash::vk::Image, format : ash::vk::Format, old : ash::vk::ImageLayout, new : ash::vk::ImageLayout)
    {

    }


    fn create_image_from_path(device : &ash::Device,  memory_properties : ash::vk::PhysicalDeviceMemoryProperties, command_pool : ash::vk::CommandPool, path : String){

        let mut png = png::Png::from_path("res/pig.png");
        let mut png_result = png.draw();
        
        let image_extent = ash::vk::Extent2D { width : png_result.width , height : png_result.height };

        let image_buffer_info =  ash::vk::BufferCreateInfo{
            size : (png_result.bpp/8) as u64,
            usage : ash::vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: ash::vk::SharingMode::EXCLUSIVE, 
            ..Default::default()
        };
        
        let image_buffer = unsafe {device.create_buffer(&image_buffer_info, None).unwrap() };
        let image_buffer_memory_req = unsafe {device.get_buffer_memory_requirements(image_buffer) };


        let image_buffer_memory_index = utils::find_memory_type(
            image_buffer_memory_req,
            memory_properties,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let image_buffer_allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: image_buffer_memory_req.size,
            memory_type_index: image_buffer_memory_index,
            ..Default::default()
        };

        let image_buffer_memory = unsafe {device.allocate_memory(&image_buffer_allocate_info, None).unwrap() };

        let image_ptr = unsafe {device.map_memory(image_buffer_memory, 0, image_buffer_memory_req.size, ash::vk::MemoryMapFlags::empty()).unwrap() };

        let mut image_slice = unsafe { Align::new( image_ptr, std::mem::align_of::<u8>() as u64, image_buffer_memory_req.size) };

        image_slice.copy_from_slice(&png_result.buf);
        unsafe { device.unmap_memory(image_buffer_memory) };
        unsafe { device.bind_buffer_memory(image_buffer, image_buffer_memory, 0).unwrap() };

        let texture_create_info = ash::vk::ImageCreateInfo {
            image_type: ash::vk::ImageType::TYPE_2D,
            format: ash::vk::Format::R8G8B8A8_UNORM,
            extent: image_extent.into(),
            mip_levels: 1,
            array_layers: 1,
            samples: ash::vk::SampleCountFlags::TYPE_1,
            tiling: ash::vk::ImageTiling::OPTIMAL,
            usage: ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
            sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let texture_image = unsafe {device.create_image(&texture_create_info, None).unwrap() };
        //* C'EST NOTRE IMAGE (image)
        let texture_memory_req = unsafe {device.get_image_memory_requirements(texture_image) };
        let texture_memory_index = utils::find_memory_type(
            texture_memory_req,
            memory_properties,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let texture_allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: texture_memory_req.size,
            memory_type_index: texture_memory_index,
            ..Default::default()
        };

        let texture_memory = unsafe {device.allocate_memory(&texture_allocate_info, None).unwrap() };
        unsafe {device.bind_image_memory(texture_image, texture_memory, 0)};
        //* C'EST NOTRE MEMOIRE  memory*/


        // TODO ICI FENCE



    
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