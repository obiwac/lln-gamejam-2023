mod aqua;

fn main() {
	let name = "Louvain-li-Nux Gamejam 2023";

	let mut win = aqua::win::Win::new(800, 600);
	win.caption(name);

	let vk_context = aqua::vk::VkContext::new(win, name, 0, 1, 0);
	let instance = &vk_context.instance;

	unsafe { instance.enumerate_physical_devices() };

	std::thread::sleep(std::time::Duration::from_millis(1000));
}
