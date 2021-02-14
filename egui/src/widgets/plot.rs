//! Simple plotting library.

#![allow(clippy::comparison_chain)]

use crate::*;

// ----------------------------------------------------------------------------

/// A value in the value-space of the plot.
/// Uses f64 for numerical stability,
/// and also to differentiate it from screen-space positions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value {
    /// This is often something monotonically increasing, such as time, but doesn't have to be.
    pub x: f64,
    pub y: f64,
}

impl Value {
    pub fn new(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

// ----------------------------------------------------------------------------

/// A horizontal line in a plot, filling the full width
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HLine {
    y: f64,
    stroke: Stroke,
}

impl HLine {
    pub fn new(y: impl Into<f64>, stroke: impl Into<Stroke>) -> Self {
        Self {
            y: y.into(),
            stroke: stroke.into(),
        }
    }
}

// ----------------------------------------------------------------------------

/// A series of values forming a path.
#[derive(Clone, PartialEq)]
pub struct Curve {
    values: Vec<Value>,
    bounds: Rect, // TODO: f64
    stroke: Stroke,
    name: String,
}

impl Curve {
    pub fn from_values(values: Vec<Value>) -> Self {
        let mut bounds = Rect::NOTHING;
        for value in &values {
            bounds.extend_with(pos2(value.x as f32, value.y as f32));
        }
        Self {
            values,
            bounds,
            stroke: Stroke::new(1.5, Color32::from_gray(120)),
            name: Default::default(),
        }
    }

    pub fn from_iter(iter: impl Iterator<Item = Value>) -> Self {
        Self::from_values(iter.collect())
    }

    /// From a series of y-values.
    /// The x-values will be the indices of these values
    pub fn from_ys_f32(ys: &[f32]) -> Self {
        let values: Vec<Value> = ys
            .iter()
            .enumerate()
            .map(|(i, &y)| Value {
                x: i as f64,
                y: y as f64,
            })
            .collect();
        Self::from_values(values)
    }

    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Stroke width (in points).
    pub fn width(mut self, width: f32) -> Self {
        self.stroke.width = width;
        self
    }

    /// Stroke color.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Name of this curve.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

// ----------------------------------------------------------------------------

/// A 2D plot, e.g. a graph of a function.
///
/// `Plot` supports multiple curves.
///
/// ```
/// # let ui = &mut Ui::__test();
/// use egui::plot::{Curve, Plot, Value};
/// let sin = (0..1000).map(|i| {
///     let x = i as f64 * 0.01;
///     Value::new(x, x.sin())
/// });
/// let curve = Curve::from_iter(sin).color(Color32::from_rgb(50, 200, 50));
/// ui.add(
///     Plot::default().curve(curve).aspect_ratio(2.0)
/// );
/// ```
#[derive(Clone, PartialEq)]
pub struct Plot {
    curves: Vec<Curve>,
    hlines: Vec<HLine>,

    bounds: Rect, // TODO: f64
    symmetrical_x_bounds: bool,
    symmetrical_y_bounds: bool,
    margin_points: Vec2,

    width: Option<f32>,
    height: Option<f32>,
    aspect_ratio: Option<f32>,

    show_x: bool,
    show_y: bool,
}

impl Default for Plot {
    fn default() -> Self {
        Self {
            curves: Default::default(),
            hlines: Default::default(),

            bounds: Rect::NOTHING,
            symmetrical_x_bounds: false,
            symmetrical_y_bounds: false,
            margin_points: vec2(4.0, 4.0),

            width: None,
            height: None,
            aspect_ratio: None,

            show_x: true,
            show_y: true,
        }
    }
}

impl Plot {
    pub fn curve(mut self, curve: Curve) -> Self {
        self.bounds = self.bounds.union(curve.bounds);
        self.curves.push(curve);
        self
    }

    /// Add a horizontal line
    pub fn hline(mut self, hline: HLine) -> Self {
        self = self.include_y(hline.y);
        self.hlines.push(hline);
        self
    }

    /// Expand bounds to include the given y value
    pub fn include_y(mut self, y: impl Into<f64>) -> Self {
        let y = y.into();
        self.bounds.min.y = self.bounds.min.y.min(y as f32);
        self.bounds.max.y = self.bounds.max.y.max(y as f32);
        self
    }

    /// If true, the x-bounds will be symmetrical, so that the x=0 zero line
    /// is always in the center.
    pub fn symmetrical_x_bounds(mut self, symmetrical_x_bounds: bool) -> Self {
        self.symmetrical_x_bounds = symmetrical_x_bounds;
        self
    }

    /// If true, the y-bounds will be symmetrical, so that the y=0 zero line
    /// is always in the center.
    pub fn symmetrical_y_bounds(mut self, symmetrical_y_bounds: bool) -> Self {
        self.symmetrical_y_bounds = symmetrical_y_bounds;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// width / height ratio.
    pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Show the x-value (e.g. when hovering). Default: `true`.
    pub fn show_x(mut self, show_x: bool) -> Self {
        self.show_x = show_x;
        self
    }

    /// Show the y-value (e.g. when hovering). Default: `true`.
    pub fn show_y(mut self, show_y: bool) -> Self {
        self.show_y = show_y;
        self
    }
}

impl Widget for Plot {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            curves,
            hlines,
            bounds,
            symmetrical_x_bounds,
            symmetrical_y_bounds,
            margin_points,
            width,
            height,
            aspect_ratio,
            show_x,
            show_y,
        } = self;

        let width = width.unwrap_or_else(|| {
            if let (Some(h), Some(aspect)) = (height, aspect_ratio) {
                h * aspect
            } else {
                ui.available_size_before_wrap_finite().x
            }
        });
        let height = height.unwrap_or_else(|| {
            if let Some(aspect) = aspect_ratio {
                width / aspect
            } else {
                ui.available_size_before_wrap_finite().y
            }
        });
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        let mut bounds = bounds;

        if symmetrical_x_bounds {
            let x_abs = bounds.max.x.abs().max(bounds.min.x.abs());
            bounds.min.x = -x_abs;
            bounds.max.x = x_abs;
        };
        if symmetrical_y_bounds {
            let y_abs = bounds.max.y.abs().max(bounds.min.y.abs());
            bounds.min.y = -y_abs;
            bounds.max.y = y_abs;
        };

        let margin_in_values = margin_points * bounds.size() / rect.size();
        let bounds = bounds.expand2(margin_in_values);

        // Background:
        ui.painter().add(Shape::Rect {
            rect,
            corner_radius: 2.0,
            fill: Color32::from_gray(10),
            stroke: Stroke::new(1.0, Color32::from_gray(120)),
        });

        if bounds.is_finite() {
            let prepared = Prepared {
                curves,
                hlines,
                to_screen: emath::RectTransform::from_to(bounds, rect),
                from_screen: emath::RectTransform::from_to(rect, bounds),
                show_x,
                show_y,
            };
            prepared.ui(ui, &response);
        }

        response
    }
}

struct Prepared {
    curves: Vec<Curve>,
    hlines: Vec<HLine>,
    to_screen: emath::RectTransform,
    from_screen: emath::RectTransform,
    show_x: bool,
    show_y: bool,
}

impl Prepared {
    fn position_from_value(&self, value: &Value) -> Pos2 {
        self.to_screen * pos2(value.x as f32, value.y as f32)
    }

    fn value_from_position(&self, pos: Pos2) -> Value {
        let v = self.from_screen * pos;
        Value::new(v.x, v.y)
    }

    /// Range of the values
    fn bounds(&self) -> &Rect {
        self.to_screen.from()
    }

    /// Where on screen we paint
    fn screen_rect(&self) -> &Rect {
        self.to_screen.to()
    }

    fn ui(&self, ui: &mut Ui, response: &Response) {
        let mut shapes = Vec::with_capacity(self.hlines.len() + self.curves.len() + 2);

        for &hline in &self.hlines {
            let HLine { y, stroke } = hline;
            let points = [
                self.position_from_value(&Value::new(self.bounds().left(), y)),
                self.position_from_value(&Value::new(self.bounds().right(), y)),
            ];
            shapes.push(Shape::line_segment(points, stroke));
        }

        for curve in &self.curves {
            let stroke = curve.stroke;
            let values = &curve.values;
            if values.len() == 1 {
                let point = self.position_from_value(&values[0]);
                shapes.push(Shape::circle_filled(
                    point,
                    stroke.width / 2.0,
                    stroke.color,
                ));
            } else if values.len() > 1 {
                shapes.push(Shape::line(
                    values.iter().map(|v| self.position_from_value(v)).collect(),
                    stroke,
                ));
            }
        }

        if response.hovered() {
            if let Some(pointer) = ui.input().pointer.tooltip_pos() {
                self.hover(ui, pointer, &mut shapes);
            }
        }

        ui.painter().sub_region(*self.screen_rect()).extend(shapes);
    }

    fn hover(&self, ui: &Ui, pointer: Pos2, shapes: &mut Vec<Shape>) {
        if !self.show_x && !self.show_y {
            return;
        }

        let interact_radius: f32 = 16.0;
        let mut closest_value = None;
        let mut closest_curve = None;
        let mut closest_dist_sq = interact_radius.powi(2);
        for curve in &self.curves {
            for value in &curve.values {
                let pos = self.position_from_value(value);
                let dist_sq = pointer.distance_sq(pos);
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_value = Some(value);
                    closest_curve = Some(curve);
                }
            }
        }

        let mut prefix = String::new();
        if let Some(curve) = closest_curve {
            if !curve.name.is_empty() {
                prefix = format!("{}\n", curve.name);
            }
        }

        let value = if let Some(value) = closest_value {
            let position = self.position_from_value(value);
            shapes.push(Shape::circle_filled(position, 3.0, Color32::WHITE));
            *value
        } else {
            self.value_from_position(pointer)
        };
        let pointer = self.position_from_value(&value);

        let rect = self.screen_rect();

        if self.show_x {
            // vertical line
            shapes.push(Shape::line_segment(
                [pos2(pointer.x, rect.top()), pos2(pointer.x, rect.bottom())],
                (1.0, Color32::from_gray(120)),
            ));
        }
        if self.show_y {
            // horizontal line
            shapes.push(Shape::line_segment(
                [pos2(rect.left(), pointer.y), pos2(rect.right(), pointer.y)],
                (1.0, Color32::from_gray(120)),
            ));
        }

        let text = {
            let d_dpixel = self.bounds().size() / self.screen_rect().size();
            let x_decimals = ((-d_dpixel.x.log10()).ceil().at_least(0.0) as usize).at_most(6);
            let y_decimals = ((-d_dpixel.y.log10()).ceil().at_least(0.0) as usize).at_most(6);
            if self.show_x && self.show_y {
                format!(
                    "{}x = {:.*}\ny = {:.*}",
                    prefix, x_decimals, value.x, y_decimals, value.y
                )
            } else if self.show_x {
                format!("{}x = {:.*}", prefix, x_decimals, value.x)
            } else if self.show_y {
                format!("{}y = {:.*}", prefix, y_decimals, value.y)
            } else {
                unreachable!()
            }
        };

        shapes.push(Shape::text(
            ui.fonts(),
            pointer,
            Align2::CENTER_BOTTOM,
            text,
            TextStyle::Body,
            Color32::WHITE,
        ));
    }
}
