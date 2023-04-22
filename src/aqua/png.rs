use aqua;

pub struct Png {
	dev: aqua::Device,
	png: u64,
}

impl Png {
	pub fn from_path(path: &str) -> Png {
		let dev = aqua::query_device("aquabsd.alps.png");

		let mut buf = std::fs::read(path).expect("can't open file");

		let png = aqua::send_device!(dev, 0x6C64, buf.as_ptr());

		Png {
			dev: dev,
			png: png,
		}
	}
}
