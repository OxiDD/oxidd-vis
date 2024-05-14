use crate::types::util::drawing::diagram_layout::Point;

use super::matrix4::Matrix4;

pub struct Transformation {
    pub angle: f32,
    pub position: Point,
    pub scale: f32,
}

impl Transformation {
    pub fn get_matrix(&self) -> Matrix4 {
        let x = self.position.x;
        let y = self.position.y;
        let transpose = Matrix4([
            1.0, 0.0, 0.0, x, //
            0.0, 1.0, 0.0, y, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let scl = self.scale;
        let scaled = Matrix4([
            scl, 0.0, 0.0, 0.0, //
            0.0, scl, 0.0, 0.0, //
            0.0, 0.0, scl, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let c = f32::cos(self.angle);
        let s = f32::sin(self.angle);
        let rotated = Matrix4([
            c, -s, 0.0, 0.0, //
            s, c, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);

        scaled.mul(&transpose).mul(&rotated)
    }
}
