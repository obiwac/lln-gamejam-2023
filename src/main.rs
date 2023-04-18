mod aqua;

fn main() {
	let mut win = aqua::win::Win::new(800, 600);
	win.caption("Louvain-li-Nux Gamejam 2023");

	let mut vk_context = aqua::vk::VkContext::new(win);

	std::thread::sleep(std::time::Duration::from_millis(3000));
}
