use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    rc::Rc,
    slice,
};

use i_float::f64_point::F64Point;
use i_overlay::core::{
    fill_rule::FillRule, float_overlay::FloatOverlay, overlay::ShapeType, overlay_rule::OverlayRule,
};
use itertools::Itertools;
use swash::{
    proxy::CharmapProxy,
    scale::{outline::Outline, Render, ScaleContext, Scaler, Source},
    shape::{ShapeContext, Shaper},
    zeno::{Command, PathData, Vector},
    Charmap, FontRef, GlyphId,
};
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::{
            render_texture::{RenderTarget, RenderTexture},
            text::triangulate::triangulate,
            vertex_renderer::VertexRenderer,
        },
    },
    util::{logging::console, matrix4::Matrix4, rectangle::Rectangle},
};

pub struct TextRenderer {
    vertex_renderer: VertexRenderer,
    char_renderer: VertexRenderer,
    char_atlas: RenderTexture,
    char_atlas_poses: HashMap<GlyphId, (Rectangle, Point)>,
    settings: TextRendererSettings,
    cur_scale: f32,
    cur_text: Vec<Text>,

    // Font helpers
    _font_data: Box<[u8]>,
    font: FontRef<'static>,
    _char_scaler_context: Box<ScaleContext>,
    char_scaler: Scaler<'static>,
}

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub position: Transition<Point>,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

/**
 * TODO:
 * - Distribute characters over multiple textures when scaling becomes too large
 */
impl TextRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        font_data: Vec<u8>,
        settings: TextRendererSettings,
    ) -> TextRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("text_renderer.vert"),
            include_str!("text_renderer.frag"),
        )
        .unwrap();
        let char_renderer = VertexRenderer::new(
            context,
            include_str!("char_renderer.vert"),
            include_str!("char_renderer.frag"),
        )
        .unwrap();

        // Every time unsafe is used, we make sure we own the data in this struct first, and don't leak any data outside
        let font_data: Box<[u8]> = font_data.into_boxed_slice();
        let font_data_ref = unsafe { std::mem::transmute::<&[u8], &'static [u8]>(&*font_data) };
        let font = FontRef::from_index(&font_data_ref[..], 0).unwrap();
        // let charmap = CharmapProxy::from_font(&font).materialize(&font);

        let mut scaler_context = Box::new(ScaleContext::new());
        let scaler_context_ref = unsafe {
            std::mem::transmute::<&mut ScaleContext, &'static mut ScaleContext>(
                scaler_context.as_mut(),
            )
        };
        let scaler = scaler_context_ref
            .builder(font)
            .size(settings.resolution * settings.scale_threshold)
            .build();

        TextRenderer {
            vertex_renderer,
            char_renderer,
            char_atlas: RenderTexture::new(context, 1, 1, (0.0, 0.0, 0.0, 0.0)).unwrap(),
            char_atlas_poses: HashMap::new(),
            _font_data: font_data,
            settings,
            cur_scale: 1.,
            cur_text: Vec::new(),

            _char_scaler_context: scaler_context,
            char_scaler: scaler,
            font,
        }
    }

    pub fn get_char_scale(&self) -> f32 {
        self.cur_scale
    }
    pub fn get_draw_scale(&self) -> f32 {
        self.settings.text_size
    }
    pub fn get_atlas_resolution(&self) -> f32 {
        self.settings.resolution * self.settings.scale_threshold
    }

    fn update_chars(&mut self, context: &WebGl2RenderingContext, required_chars: Vec<GlyphId>) {
        let count = required_chars.len() * 2;
        let chars = required_chars
            .iter()
            .cloned()
            .chain(self.char_atlas_poses.keys().cloned())
            .take(count)
            .collect::<Vec<GlyphId>>();

        let char_data = chars
            .iter()
            .filter_map(|&glyph_id| {
                let outline = self.char_scaler.scale_outline(glyph_id)?;
                let bounds = outline.bounds();
                let width = bounds.width() * self.get_char_scale();
                let height = bounds.height() * self.get_char_scale();
                if width == 0.0 || height == 0.0 {
                    None
                } else {
                    Some((glyph_id, ((width, height), outline)))
                }
            })
            .collect();

        self.position_chars(context, &char_data);
        self.draw_chars(context, &char_data);
    }

    fn position_chars(&mut self, context: &WebGl2RenderingContext, char_data: &CharDataMap) {
        let area = char_data
            .values()
            .fold(0., |sum, ((width, height), _)| sum + width * height);
        let target_width = area.sqrt();

        self.char_atlas_poses.clear();

        let mut width = 0.;
        let mut row_height = 0.;
        let mut x = 0.;
        let mut y = 0.;
        for char in char_data.keys() {
            let Some(((char_width, char_height), outline)) = char_data.get(char) else {
                continue;
            };
            let min = outline.bounds().min * self.get_char_scale();
            self.char_atlas_poses.insert(
                *char,
                (
                    Rectangle::new(x, y, *char_width, *char_height),
                    Point { x: min.x, y: min.y },
                ),
            );
            x += *char_width + self.settings.atlas_spacing;
            if *char_height > row_height {
                row_height = *char_height;
            }

            if x > width {
                width = x;
            }
            if x > target_width {
                x = 0.;
                y += row_height + self.settings.atlas_spacing;
                row_height = 0.;
            }
        }

        let width = f32::ceil(width) as usize;
        let height = f32::ceil(y + row_height) as usize;
        console::log!("atlas: ({}, {})", width, height);
        self.char_atlas.dispose(context);
        self.char_atlas = RenderTexture::new(context, width, height, (0.0, 0.0, 0.0, 0.0)).unwrap();
    }

    fn get_atlas_coord(&self, point: Point) -> Point {
        let texture_size = self.char_atlas.get_size();
        Point {
            x: point.x / (texture_size.0 as f32),
            y: point.y / (texture_size.1 as f32),
        }
    }

    fn draw_chars(&mut self, context: &WebGl2RenderingContext, char_data: &CharDataMap) {
        let glyphs = char_data.iter().filter_map(|(glyph_id, (_, outline))| {
            self.char_atlas_poses
                .get(glyph_id)
                .map(|pos| (outline, pos))
        });

        let texture_size = self.char_atlas.get_size();
        let texture_size = Point {
            x: texture_size.0 as f32,
            y: texture_size.1 as f32,
        };
        self.char_renderer.set_data(
            context,
            "position",
            &glyphs
                .flat_map(|(outline, (pos, min))| {
                    let offset = pos.pos() - *min;

                    let scale = self.get_char_scale();
                    let display_scale = self.get_draw_scale() * scale;
                    // display_scale == 1 when the character size is self.resolution, and gets smaller as the character gets smaller. Hence we can use more pixels per character then, before scaling of the character, however we should not do this linearly or we lose too much detail on small scale, hence we take the sqrt to preserve more detail even at small scale.
                    let distance_per_sample = self.settings.sample_distance / display_scale.sqrt();
                    let triangles = triangulate(outline.path().commands(), distance_per_sample);
                    triangles
                        .iter()
                        .flat_map(|point| {
                            vec![
                                (point.x * scale + offset.x) / texture_size.x,
                                (point.y * scale + offset.y) / texture_size.y,
                            ]
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<f32>>(),
            2,
        );

        self.char_renderer.update_data(context);

        self.char_atlas.bind_buffer(context);
        self.char_atlas.clear(context);
        self.char_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn set_texts(&mut self, context: &WebGl2RenderingContext, texts: &Vec<Text>) {
        self.cur_text = texts.clone();

        // Obtain the character glyphs and position data, and ensure that these glyphs are on the atlas
        let char_data = texts
            .iter()
            .flat_map(|text| {
                let mut shaper_context = Box::new(ShapeContext::new());
                let mut shaper = shaper_context
                    .builder(self.font)
                    .size(self.settings.text_size)
                    .build();

                shaper.add_str(&text.text);
                let mut chars = Vec::new();
                let mut x = 0.;
                shaper.shape_with(|cluster| {
                    for glyph in cluster.glyphs {
                        chars.push((
                            glyph.id,
                            text.position
                                + Transition::plain(Point {
                                    x: glyph.x + x,
                                    y: glyph.y,
                                }),
                        ));
                        x += glyph.advance;
                    }
                });
                chars
            })
            .collect::<Vec<_>>();
        let mut glyphs = char_data
            .iter()
            .map(|&(glyph_id, _)| glyph_id)
            .collect::<HashSet<GlyphId>>();

        let charmap = self.font.charmap();
        for char in self.settings.default_chars.chars() {
            glyphs.insert(charmap.map(char));
        }

        let has_new_glyphs = !glyphs.is_subset(&self.char_atlas_poses.keys().cloned().collect());
        if has_new_glyphs {
            self.update_chars(context, glyphs.iter().cloned().collect());
        }

        // Bind the character data to the shader
        let char_data = char_data
            .iter()
            .filter_map(|(glyph_id, pos)| {
                self.char_atlas_poses
                    .get(glyph_id)
                    .map(|glyph_pos| (glyph_pos, pos))
            })
            .collect::<Vec<_>>();

        let make_square = |p: Point, size: Point| {
            let p1 = p;
            let p2 = p + Point { x: 0., y: size.y };
            let p3 = p + size;
            let p4 = p + Point { x: size.x, y: 0. };
            [
                p1.x, p1.y, //
                p2.x, p2.y, //
                p4.x, p4.y, //
                /* */
                p3.x, p3.y, //
                p2.x, p2.y, //
                p4.x, p4.y,
            ]
        };

        let char_to_draw_scale =
            self.get_draw_scale() / self.get_char_scale() / self.get_atlas_resolution();

        let positions_old = char_data
            .iter()
            .flat_map(|((glyph_pos, offset), text_pos)| {
                make_square(
                    text_pos.old + *offset * char_to_draw_scale,
                    glyph_pos.size() * char_to_draw_scale,
                )
            })
            .collect::<Vec<_>>();

        let positions_new = char_data
            .iter()
            .flat_map(|((glyph_pos, offset), text_pos)| {
                make_square(
                    text_pos.new + *offset * char_to_draw_scale,
                    glyph_pos.size() * char_to_draw_scale,
                )
            })
            .collect::<Vec<_>>();

        let positions_start_time = char_data
            .iter()
            .flat_map(|(_, text_pos)| [text_pos.old_time as f32; 6])
            .collect::<Vec<_>>();
        let positions_duration = char_data
            .iter()
            .flat_map(|(_, text_pos)| [text_pos.duration as f32; 6])
            .collect::<Vec<_>>();

        let char_coords = char_data
            .iter()
            .flat_map(|((glyph_pos, _), _)| {
                make_square(
                    self.get_atlas_coord(glyph_pos.pos()),
                    self.get_atlas_coord(glyph_pos.size()),
                )
            })
            .collect::<Vec<_>>();

        self.vertex_renderer
            .set_data(context, "positionOld", &positions_old, 2);
        self.vertex_renderer
            .set_data(context, "position", &positions_new, 2);
        self.vertex_renderer
            .set_data(context, "positionStartTime", &positions_start_time, 1);
        self.vertex_renderer
            .set_data(context, "positionDuration", &positions_duration, 1);
        self.vertex_renderer
            .set_data(context, "charCoord", &char_coords, 2);

        self.vertex_renderer.update_data(context);
    }

    pub fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        let p1 = transform.mul_vec3((0., 0., 0.));
        let p2 = transform.mul_vec3((1., 0., 0.));
        let scale = Point { x: p1.0, y: p1.1 }
            .distance(&Point { x: p2.0, y: p2.1 })
            .min(self.settings.max_scale);

        let factor = 3.;
        if scale > self.cur_scale * factor || self.cur_scale > scale * factor {
            self.cur_scale = scale;
            // self.cur_scale = 0.07;
            // console::log!("{}", scale);
            self.update_chars(context, self.char_atlas_poses.keys().cloned().collect());
            self.set_texts(context, &self.cur_text.clone());
        }
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    pub fn render<T: RenderTarget>(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        target: &T,
    ) {
        target.bind_buffer(context);
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));

        let atlas = &self.char_atlas;
        self.vertex_renderer
            .set_uniform(context, "characters", |u| {
                atlas.bind_texture(context, u, 0);
            });

        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
        self.char_renderer.dispose(context);
        self.char_atlas.dispose(context);
    }
}

type CharDataMap = HashMap<u16, ((f32, f32), Outline)>;

pub struct TextRendererSettings {
    /// The screen height to base the atlas resolution on
    pub resolution: f32,
    /// The factor difference allowed in scale before rescaling the atlas
    pub scale_threshold: f32,
    /// The relative screen size to render the text at
    pub text_size: f32,
    /// The sample distance between points expressed in relation to the resolution (i.e. in terms of the distance between pixels if a character is rendered at full resolution)
    pub sample_distance: f32,
    /// The maximum rendering scale to use for the atlas, above which rendering quality won't be increased anymore
    pub max_scale: f32,
    /// The spacing that characters have in the atlas
    pub atlas_spacing: f32,
    /// The default characters that should always be included on the atlas
    pub default_chars: String,
}

impl TextRendererSettings {
    pub fn new() -> TextRendererSettings {
        TextRendererSettings {
            resolution: 1080.,
            scale_threshold: 2.0,
            text_size: 1.0,
            sample_distance: 25.,
            max_scale: 1.0,
            atlas_spacing: 2.0,
            default_chars: "abcdefghijklmnopqrstuvwxyz_-".to_string(),
        }
    }
    pub fn resolution(mut self, resolution: f32) -> TextRendererSettings {
        self.resolution = resolution;
        self
    }
    pub fn scale_threshold(mut self, scale_threshold: f32) -> TextRendererSettings {
        self.scale_threshold = scale_threshold;
        self
    }
    pub fn text_size(mut self, text_size: f32) -> TextRendererSettings {
        self.text_size = text_size;
        self
    }
    pub fn sample_distance(mut self, sample_distance: f32) -> TextRendererSettings {
        self.sample_distance = sample_distance;
        self
    }
    pub fn max_scale(mut self, max_scale: f32) -> TextRendererSettings {
        self.max_scale = max_scale;
        self
    }
    pub fn atlas_spacing(mut self, atlas_spacing: f32) -> TextRendererSettings {
        self.atlas_spacing = atlas_spacing;
        self
    }
    pub fn default_chars(mut self, default_chars: String) -> TextRendererSettings {
        self.default_chars = default_chars;
        self
    }
}
