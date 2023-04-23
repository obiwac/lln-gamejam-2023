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
	pub mv_matrix: Mat,
	pub p_matrix: Mat,
}

impl Player {
	pub fn new() -> Player {
		Player {
			mv_matrix: Mat::new(),
			p_matrix: Mat::new(),
		}
	}
}
