#[path = "utils.rs"] mod utils;
extern crate ash;

use std::mem::align_of;
use ash::util::*;
use std::default::Default;

use std ::{
	error::Error,
};


pub struct Indexbuffer{
    indexBuffer : ash::vk::Buffer,
    indexBufferMemory : ash::vk::DeviceMemory,
}

pub struct Vertexbuffer{
    vertexBuffer : ash::vk::Buffer,
    vertexMemory : ash::vk::DeviceMemory,
}

impl Vertexbuffer{
    
}

impl Indexbuffer{
    //TODO REMOVE PUB
    pub fn new(device : &ash::Device, memory_properties : ash::vk::PhysicalDeviceMemoryProperties, indices : std::vec::Vec<u32> ) -> Result<Indexbuffer,
    Box<dyn Error>> {
        let buffer_info = ash::vk::BufferCreateInfo::default()
            .size(std::mem::size_of_val(&indices) as u64)
            .usage(ash::vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
        let index_buffer = unsafe { device.create_buffer(&buffer_info, None).unwrap() };
        let index_buffer_memory_req = unsafe { device.get_buffer_memory_requirements(index_buffer) };
        // In textures
        let index_buffer_memory_index = utils::find_memory_type(
            index_buffer_memory_req,
            memory_properties, // Passed in main
           ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        
        let index_allocate_info =   ash::vk::MemoryAllocateInfo {
            allocation_size: index_buffer_memory_req.size,
            memory_type_index: index_buffer_memory_index,
            ..Default::default()
        };

        let index_buffer_memory = unsafe { device.allocate_memory(&index_allocate_info, None).unwrap() };
        let index_ptr = unsafe { device.map_memory(index_buffer_memory, 0, index_buffer_memory_req.size, ash::vk::MemoryMapFlags::empty()).unwrap() };
        let mut index_slice = unsafe { Align::new( index_ptr, align_of::<u32>() as u64, index_buffer_memory_req.size ) };
        
        index_slice.copy_from_slice(&indices);
        unsafe { device.unmap_memory(index_buffer_memory) };
        unsafe { device.bind_buffer_memory(index_buffer, index_buffer_memory, 0).unwrap() };

        Ok(Indexbuffer{
            indexBuffer : index_buffer, 
            indexBufferMemory : index_buffer_memory,
        })
                
    }
}