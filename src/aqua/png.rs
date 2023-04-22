use aqua;

pub struct Png {
	dev: aqua::Device,
	png: u64,
}

pub struct PngResult {
	pub buf: Vec<u8>,

	pub bpp: u32,
	pub width: u32,
	pub height: u32,
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

	pub fn draw(&mut self) -> PngResult {
		let mut c_buf = 0u64;

		let mut c_bpp = 0u64;
		let mut c_width = 0u64;
		let mut c_height = 0u64;

		aqua::send_device!(self.dev, 0x6477, self.png,
			&mut c_buf as *mut _,

			&mut c_bpp as *mut _,
			&mut c_width as *mut _,
			&mut c_height as *mut _
		);

		let len = (c_width * c_height * c_bpp / 8) as usize;
		let buf = unsafe { Vec::from_raw_parts(std::mem::transmute(c_buf), len, len) };

		PngResult {
			buf: buf,

			bpp: c_bpp as u32,
			width: c_width as u32,
			height: c_height as u32,
		}
	}
}
