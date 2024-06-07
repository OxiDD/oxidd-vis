use std::fmt::Display;

use crate::types::util::drawing::diagram_layout::Point;

use super::matrix4::Matrix4;

#[derive(Clone)]
pub struct Transformation {
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub position: Point,
    pub scale: f32,
}
impl Transformation {
    pub fn default() -> Transformation {
        Transformation {
            width: 1.0,
            height: 1.0,
            angle: 0.0,
            position: Point { x: 0.0, y: 0.0 },
            scale: 1.0,
        }
    }
    pub fn get_matrix(&self) -> Matrix4 {
        let asx = 1.0 / self.width;
        let asy = 1.0 / self.height;
        let aspected = Matrix4([
            asx, 0.0, 0.0, 0.0, //
            0.0, asy, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
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

        aspected.mul(&scaled).mul(&transpose).mul(&rotated)
    }

    pub fn get_inverse_matrix(&self) -> Matrix4 {
        let asx = self.width;
        let asy = self.height;
        let aspected = Matrix4([
            asx, 0.0, 0.0, 0.0, //
            0.0, asy, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let x = -self.position.x;
        let y = -self.position.y;
        let transpose = Matrix4([
            1.0, 0.0, 0.0, x, //
            0.0, 1.0, 0.0, y, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let scl = 1.0 / self.scale;
        let scaled = Matrix4([
            scl, 0.0, 0.0, 0.0, //
            0.0, scl, 0.0, 0.0, //
            0.0, 0.0, scl, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let c = f32::cos(-self.angle);
        let s = f32::sin(-self.angle);
        let rotated = Matrix4([
            c, -s, 0.0, 0.0, //
            s, c, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);

        rotated.mul(&transpose).mul(&scaled).mul(&aspected)
    }
}
