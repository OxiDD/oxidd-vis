use std::fmt::Display;

#[derive(Debug)]
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

impl Matrix4 {
    pub fn identity() {
        Matrix4([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
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
