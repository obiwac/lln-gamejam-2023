extern crate ash;

pub fn find_memory_type(
    requirements: ash::vk::MemoryRequirements,
    mem_properties: ash::vk::PhysicalDeviceMemoryProperties,
    required_properties: ash::vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type :(.")
}