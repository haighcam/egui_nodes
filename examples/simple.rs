use eframe::{egui, epi};
use egui_nodes::{Context, LinkArgs, NodeArgs, NodeConstructor, PinArgs, PinShape};

struct MyApp {
    ctx: Context,
    links: Vec<(usize, usize)>,
}

pub fn example_graph(ctx: &mut Context, links: &mut Vec<(usize, usize)>, ui: &mut egui::Ui) {
    // add nodes with attributes
    let nodes = vec![
        NodeConstructor::new(
            0,
            NodeArgs {
                outline: Some(egui::Color32::LIGHT_BLUE),
                ..Default::default()
            },
        )
        .with_origin([50.0, 150.0].into())
        .with_title(|ui| ui.label("Example Node A"))
        .with_input_attribute(
            0,
            PinArgs {
                shape: PinShape::Triangle,
                ..Default::default()
            },
            |ui| ui.label("Input"),
        )
        .with_static_attribute(1, |ui| ui.label("Can't Connect to Me"))
        .with_output_attribute(
            2,
            PinArgs {
                shape: PinShape::TriangleFilled,
                ..Default::default()
            },
            |ui| ui.label("Output"),
        ),
        NodeConstructor::new(1, Default::default())
            .with_origin([225.0, 150.0].into())
            .with_title(|ui| ui.label("Example Node B"))
            .with_static_attribute(3, |ui| ui.label("Can't Connect to Me"))
            .with_output_attribute(4, Default::default(), |ui| ui.label("Output"))
            .with_input_attribute(5, Default::default(), |ui| ui.label("Input")),
    ];

    ctx.show(
        nodes,
        links.iter().enumerate().map(|(i, (start, end))| (i, *start, *end, LinkArgs::default())),
        ui,
    );

    // remove destroyed links
    if let Some(idx) = ctx.link_destroyed() {
        links.remove(idx);
    }

    // add created links
    if let Some((start, end, _)) = ctx.link_created() {
        links.push((start, end))
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            ctx: Context::default(),
            links: Vec::new(),
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            example_graph(&mut self.ctx, &mut self.links, ui);
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}

fn main() {
    eframe::run_native(Box::new(MyApp::default()), eframe::NativeOptions::default());
}
