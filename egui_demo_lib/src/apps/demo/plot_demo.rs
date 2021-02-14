use egui::*;

pub struct PlotDemo {
    animate: bool,
}

impl Default for PlotDemo {
    fn default() -> Self {
        Self { animate: true }
    }
}

impl super::Demo for PlotDemo {
    fn name(&self) -> &str {
        "🗠 Plot"
    }

    fn show(&mut self, ctx: &CtxRef, open: &mut bool) {
        use super::View;
        Window::new(self.name())
            .open(open)
            .default_size(vec2(512.0, 256.0))
            .scroll(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for PlotDemo {
    fn ui(&mut self, ui: &mut Ui) {
        use egui::plot::{Curve, Plot, Value};
        use std::f64::consts::TAU;

        ui.checkbox(&mut self.animate, "animate");
        let time = if self.animate {
            ui.ctx().request_repaint();
            ui.input().time
        } else {
            0.0
        };

        let n = 500;
        let circle = (0..=n).map(|i| {
            let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
            let r = 0.5;
            Value::new(r * t.cos(), r * t.sin())
        });
        let circle = Curve::from_iter(circle)
            .color(Color32::from_rgb(100, 240, 100))
            .name("circle");

        let n = 5_000;
        let curve = (0..=n).map(|i| {
            let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
            Value::new((4.0 * t + time).sin(), (6.0 * t).sin())
        });
        let curve = Curve::from_iter(curve)
            .color(Color32::from_rgb(100, 150, 250))
            .name("x=sin(4t), y=sin(6t)");

        ui.add(Plot::default().curve(circle).curve(curve).aspect_ratio(1.0));
    }
}
