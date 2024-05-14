use oxidd_core::Tag;

use super::diagram_layout::DiagramLayout;

/// A trait for rendering a given layout
pub trait Renderer<T: Tag> {
    fn set_transform(&mut self, x: f32, y: f32, scale: f32);
    fn update_layout(&mut self, layout: &DiagramLayout<T>);
    fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]);
}
