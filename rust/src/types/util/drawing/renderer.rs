use oxidd_core::Tag;

use super::diagram_layout::DiagramLayout;

/// A trait for rendering a given layout
pub trait Renderer<T: Tag> {
    fn render(
        &self,
        layout: &DiagramLayout<T>,
        time: i32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    );
}
