#[path = "utils.rs"] mod utils;
extern crate ash;
struct Texture {
    pub image:  ash::vk::Image,
    pub memory: ash::vk::DeviceMemory,
    pub view: ash::vk::ImageView,
    pub sampler: Option<ash::vk::Sampler>,
}



impl Texture {
    fn create_image_from_path(path : String){
       
    }
    fn create_image( device : &ash::Device, memory_properties : ash::vk::PhysicalDeviceMemoryProperties, memory_properties_flag : ash::vk::MemoryPropertyFlags, 
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