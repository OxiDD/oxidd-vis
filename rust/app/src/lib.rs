mod components;
mod configuration;
mod inputs;
mod new_wasm_interface;
mod traits;
mod types;
mod util;
mod wasm_interface;

use util::panic_hook::set_panic_hook;
use wasm_bindgen::prelude::*;

use types::{mtbdd::mtbdd_drawer::MTBDDDiagram, qdd::qdd_drawer::QDDDiagram};

use crate::{
    components::{
        button_component::ButtonComp, ComponentWithData, CompositeComp, ContainerComp, FillComp,
        LabelComp, LabelKind, OverlayComp, PanelButtonComp, PanelComp, PositionOverlayComp,
        TooltipComp,
    },
    inputs::string_input::{StringInput, StringInputComp},
    util::watchables::{ClonableWatchableUtils, Field, StringField, WatchableUtils},
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_qdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(QDDDiagram::new())))
}

#[wasm_bindgen]
pub fn create_mtbdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(MTBDDDiagram::new())))
}

#[wasm_bindgen]
pub fn test_panel() -> PanelComp {
    let text = StringInput::from("test");
    let mut text_clone = text.clone();
    let text_clone2 = text.clone();

    let button = ButtonComp::builder()
        .icon("AlarmClock")
        .text(text.map(|v| Some(format!("Button: {v}"))))
        .build();
    let reset_button = ComponentWithData::new(
        button.clone(),
        button.on_click(move || {
            text_clone.set("reset".to_string());
        }),
    );

    let description = Field::new("hoi".to_string());
    let labeled_button = TooltipComp::builder()
        .tooltip(CompositeComp::builder().horizontal(true).build((
            "Something",
            ButtonComp::builder().icon("AlarmClock").build(),
        )))
        .build(
            LabelComp::builder()
                .kind(LabelKind::Inline)
                .label(description)
                .build(reset_button),
        );

    let text_field = StringInputComp::builder()
        .data(text_clone2)
        .multiline(true)
        .multiline_dynamic(true)
        .multiline_min(2)
        .late_submit(true)
        .build();

    let extra_panel_name = StringField::from("test panel");
    let extra_panel = PanelButtonComp::builder()
        .button(ButtonComp::builder().text("open panel").build())
        .panel(
            PanelComp::builder()
                .name(extra_panel_name)
                .id("extra-panel"),
        )
        .build("Some content of panel");

    let composite = (labeled_button, text_field, extra_panel);
    // let composite = CompositeComp::builder().gap(1.0).build(composite);

    let with_overlay = FillComp::new(ContainerComp::builder().padding(1.0).build(FillComp::new(
        PositionOverlayComp::bottom_right(
            ButtonComp::builder().icon("AlarmClock").build(),
            composite,
        ),
    )));

    let panel_name = StringField::from("test panel");
    PanelComp::builder()
        .name(panel_name)
        .id("test-panel")
        .open_count(0)
        .build(with_overlay)
}
