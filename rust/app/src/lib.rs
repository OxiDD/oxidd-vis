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
        button_component::ButtonComp, Align, AlignMain, ComponentWithData, CompositeComp,
        ContainerComp, FillComp, LabelComp, LabelKind, OverlayComp, PanelButtonComp, PanelComp,
        PromptComp, TooltipComp,
    },
    inputs::{
        bool_input::{BoolInput, BoolInputComp},
        f32_input::{F32Input, F32InputClamped, F32InputComp},
        string_input::{StringInput, StringInputComp},
        variant_input::{VariantComponentMapper, VariantInput, VariantInputComp, VariantOptions},
    },
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

    let text_field = StringInputComp::builder(text_clone2)
        .multiline(true)
        .multiline_dynamic(true)
        .multiline_min(2)
        .late_submit(true)
        .build();

    #[derive(PartialEq, Eq, Clone)]
    enum Option {
        V1,
        V2,
        V3,
    }
    impl VariantOptions for Option {
        fn get_variants() -> Vec<Self> {
            vec![Option::V1, Option::V2, Option::V3]
        }
    }
    let variant = VariantInput::new(Option::V1);
    let variant_comp1 = VariantInputComp::builder()
        .data(VariantComponentMapper::new(variant.clone(), |v| {
            (match v {
                Option::V1 => "V1",
                Option::V2 => "V2",
                Option::V3 => "V3",
            })
            .into()
        }))
        .build();
    let variant_comp2 = VariantInputComp::builder()
        .data(VariantComponentMapper::new(variant.clone(), |v| {
            ButtonComp::builder()
                .icon(match v {
                    Option::V1 => "PageLink",
                    Option::V2 => "CommentSolid",
                    Option::V3 => "Installation",
                })
                .text("Some label")
                .build()
                .into()
        }))
        .horizontal(false)
        .build();

    let v1_field = StringInputComp::builder(StringInput::new("V1".into())).build();
    let v2_field = StringInputComp::builder(StringInput::new("V2".into())).build();
    let v3_field = StringInputComp::builder(StringInput::new("V3".into())).build();
    let variant_comp3 = VariantInputComp::builder()
        .data(VariantComponentMapper::new(variant.clone(), move |v| {
            LabelComp::builder()
                .label(match v {
                    Option::V1 => "V1",
                    Option::V2 => "V2",
                    Option::V3 => "V3",
                })
                .build(match v {
                    Option::V1 => v1_field.clone(),
                    Option::V2 => v2_field.clone(),
                    Option::V3 => v3_field.clone(),
                })
                .into()
        }))
        .main_align(AlignMain::Stretch)
        .gap(1.0)
        .horizontal(true)
        .build();

    let extra_panel_name = StringField::from("test panel");
    let extra_panel = PanelButtonComp::builder()
        .button(ButtonComp::builder().text("open panel").build())
        .panel(
            PanelComp::builder()
                .name(extra_panel_name)
                .id("extra-panel"),
        )
        .build(
            CompositeComp::builder()
                .perpendicular_align(Align::End)
                .build((
                    F32InputComp::builder(
                        F32InputClamped::builder(F32Input::new(0.0))
                            .min(-5.0)
                            .max(5.0)
                            .precision(0.5)
                            .build(),
                    )
                    .step_size(1.0)
                    .step_round(true)
                    .build(),
                    variant_comp1,
                    variant_comp2,
                    variant_comp3,
                )),
        );

    let composite = (labeled_button, text_field, extra_panel);
    // let composite = CompositeComp::builder().gap(1.0).build(composite);

    let prompt_button = ButtonComp::builder().text("Some prompt").build();
    let prompt = (
        prompt_button.clone(),
        PromptComp::builder()
            .open_count(prompt_button.clicks())
            .build(("Hallo prompt", BoolInput::new(false))),
    );

    let with_overlay = FillComp::new(ContainerComp::builder().padding(1.0).build(FillComp::new((
        composite,
        prompt,
        OverlayComp::bottom_right(ButtonComp::builder().icon("AlarmClock").build()),
    ))));

    let panel_name = StringField::from("test panel");
    PanelComp::builder()
        .name(panel_name)
        .id("test-panel")
        .open_count(0)
        .build(with_overlay)
}
