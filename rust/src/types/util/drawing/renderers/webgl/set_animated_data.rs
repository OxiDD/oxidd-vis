use web_sys::WebGl2RenderingContext;

use crate::types::util::drawing::diagram_layout::Transition;

use super::vertex_renderer::VertexRenderer;

pub fn set_animated_data<
    const L: usize,
    T,
    I: Iterator<Item = Transition<T>> + Clone,
    V: Fn(T) -> [f32; L],
>(
    name: &str,
    data: I,
    values: V,
    context: &WebGl2RenderingContext,
    renderer: &mut VertexRenderer,
) {
    let old_values: Box<[f32]> = data.clone().flat_map(|val| values(val.old)).collect();
    renderer.set_data(context, &format!("{}Old", name)[..], &old_values, L as u8);

    let values: Box<[f32]> = data.clone().flat_map(|val| values(val.new)).collect();
    renderer.set_data(context, name, &values, L as u8);

    let value_old_times: Box<[f32]> = data.clone().flat_map(|val| [val.old_time as f32]).collect();
    renderer.set_data(
        context,
        &format!("{}StartTime", name)[..],
        &value_old_times,
        1,
    );

    let value_durations: Box<[f32]> = data.flat_map(|val| [val.duration as f32]).collect();
    renderer.set_data(
        context,
        &format!("{}Duration", name)[..],
        &value_durations,
        1,
    );
}
