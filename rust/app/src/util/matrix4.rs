use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Matrix4(pub [f32; 16]);

impl Display for Matrix4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Matrix({})",
            self.0
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{}{}", (if i % 4 == 0 { "\n" } else { "" }), v.to_string()))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

type Vec3 = (f32, f32, f32);
type Vec4 = (f32, f32, f32, f32);
impl Matrix4 {
    pub fn identity() {
        Matrix4([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
    }
    pub fn mul_vec3(&self, (v0, v1, v2): Vec3) -> Vec3 {
        let (o0, o1, o2, o3) = self.mul_vec((v0, v1, v2, 1.0));
        (o0 / o3, o1 / o3, o2 / o3)
    }
    pub fn mul_vec(&self, vec: Vec4) -> Vec4 {
        (
            self.0[0] * vec.0 + self.0[1] * vec.1 + self.0[2] * vec.2 + self.0[3] * vec.3,
            self.0[4] * vec.0 + self.0[5] * vec.1 + self.0[6] * vec.2 + self.0[7] * vec.3,
            self.0[8] * vec.0 + self.0[9] * vec.1 + self.0[10] * vec.2 + self.0[11] * vec.3,
            self.0[12] * vec.0 + self.0[13] * vec.1 + self.0[14] * vec.2 + self.0[15] * vec.3,
        )
    }
    pub fn mul(&self, other: &Matrix4) -> Matrix4 {
        Matrix4([
            self.0[0] * other.0[0]
                + self.0[1] * other.0[4]
                + self.0[2] * other.0[8]
                + self.0[3] * other.0[12],
            self.0[0] * other.0[1]
                + self.0[1] * other.0[5]
                + self.0[2] * other.0[9]
                + self.0[3] * other.0[13],
            self.0[0] * other.0[2]
                + self.0[1] * other.0[6]
                + self.0[2] * other.0[10]
                + self.0[3] * other.0[14],
            self.0[0] * other.0[3]
                + self.0[1] * other.0[7]
                + self.0[2] * other.0[11]
                + self.0[3] * other.0[15],
            self.0[4] * other.0[0]
                + self.0[5] * other.0[4]
                + self.0[6] * other.0[8]
                + self.0[7] * other.0[12],
            self.0[4] * other.0[1]
                + self.0[5] * other.0[5]
                + self.0[6] * other.0[9]
                + self.0[7] * other.0[13],
            self.0[4] * other.0[2]
                + self.0[5] * other.0[6]
                + self.0[6] * other.0[10]
                + self.0[7] * other.0[14],
            self.0[4] * other.0[3]
                + self.0[5] * other.0[7]
                + self.0[6] * other.0[11]
                + self.0[7] * other.0[15],
            self.0[8] * other.0[0]
                + self.0[9] * other.0[4]
                + self.0[10] * other.0[8]
                + self.0[11] * other.0[12],
            self.0[8] * other.0[1]
                + self.0[9] * other.0[5]
                + self.0[10] * other.0[9]
                + self.0[11] * other.0[13],
            self.0[8] * other.0[2]
                + self.0[9] * other.0[6]
                + self.0[10] * other.0[10]
                + self.0[11] * other.0[14],
            self.0[8] * other.0[3]
                + self.0[9] * other.0[7]
                + self.0[10] * other.0[11]
                + self.0[11] * other.0[15],
            self.0[12] * other.0[0]
                + self.0[13] * other.0[4]
                + self.0[14] * other.0[8]
                + self.0[15] * other.0[12],
            self.0[12] * other.0[1]
                + self.0[13] * other.0[5]
                + self.0[14] * other.0[9]
                + self.0[15] * other.0[13],
            self.0[12] * other.0[2]
                + self.0[13] * other.0[6]
                + self.0[14] * other.0[10]
                + self.0[15] * other.0[14],
            self.0[12] * other.0[3]
                + self.0[13] * other.0[7]
                + self.0[14] * other.0[11]
                + self.0[15] * other.0[15],
        ])
    }
}
