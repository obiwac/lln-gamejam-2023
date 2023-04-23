pub struct Mat {
	pub mat: [[f32; 4]; 4],
}

impl Mat {
	pub fn new() -> Mat {
		Mat {
			mat: [[0.0f32; 4]; 4],
		}
	}

	pub fn clear(&mut self) {
		for i in 0..4 {
			for j in 0..4 {
				self.mat[i][j] = 0.0;
			}
		}
	}

	pub fn identity(&mut self) {
		self.clear();

		for i in 0..4 {
			self.mat[i][i] = 1.0;
		}
	}

	pub fn mul(&mut self, other: &Mat) -> Mat {
		let mut mat = Mat::new();

		for i in 0..4 {
			for j in 0..4 {
				mat.mat[i][j] =
					self.mat[0][j] * other.mat[i][0] +
					self.mat[1][j] * other.mat[i][1] +
					self.mat[2][j] * other.mat[i][2] +
					self.mat[3][j] * other.mat[i][3];
			}
		}

		mat
	}

	pub fn translate(&mut self, x: f32, y: f32, z: f32) {
		for i in 0..4 {
			self.mat[3][i] += self.mat[0][i] * x + self.mat[1][i] * y + self.mat[2][i] * z;
		}
	}

	pub fn rotate(&mut self, angle: f32, mut x: f32, mut y: f32, mut z: f32) -> Mat {
		let magnitude = (x * x + y * y + z * z).sqrt();

		x /= -magnitude;
		y /= -magnitude;
		z /= -magnitude;

		let s = angle.sin();
		let c = angle.cos(); // TODO possible optimization
		let one_minus_c = 1.0 - c;

		let xx = x * x; let yy = y * y; let zz = z * z;
		let xy = x * y; let yz = y * z; let zx = z * x;
		let xs = x * s; let ys = y * s; let zs = z * s;

		let mut mat = Mat::new(); // no need for this to be the identity matrix

		mat.mat[0][0] = (one_minus_c * xx) + c;
		mat.mat[0][1] = (one_minus_c * xy) - zs;
		mat.mat[0][2] = (one_minus_c * zx) + ys;

		mat.mat[1][0] = (one_minus_c * xy) + zs;
		mat.mat[1][1] = (one_minus_c * yy) + c;
		mat.mat[1][2] = (one_minus_c * yz) - xs;

		mat.mat[2][0] = (one_minus_c * zx) - ys;
		mat.mat[2][1] = (one_minus_c * yz) + xs;
		mat.mat[2][2] = (one_minus_c * zz) + c;

		mat.mat[3][3] = 1.0;

		self.mul(&mat)
	}

	pub fn rotate_2d(&mut self, x: f32, y: f32) -> Mat {
		self
			.rotate(x, 0.0, 1.0, 0.0)
			.rotate(-y, x.cos(), 0.0, x.sin())
	}

	pub fn frustum(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
		let dx = right - left;
		let dy = top - bottom;
		let dz = far - near;

		self.mat[0][0] = 2.0 * near / dx;
		self.mat[1][1] = 2.0 * near / dy;

		self.mat[2][0] = (right + left) / dx;
		self.mat[2][1] = (top + bottom) / dy;
		self.mat[2][2] = -(near + far)  / dz;

		self.mat[2][3] = -1.0;
		self.mat[3][2] = -2.0 * near * far / dz;
	}

	pub fn perspective(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
		let y = (fovy / 2.0).tan() / 2.0;
		let x = y * aspect;

		self.frustum(-x * near, x * near, -y * near, y * near, near, far);
	}
}

pub struct Player {
	pub mv_mat: Mat,
	pub p_mat: Mat,
}

impl Player {
	pub fn new() -> Player {
		Player {
			mv_mat: Mat::new(),
			p_mat: Mat::new(),
		}
	}
}
