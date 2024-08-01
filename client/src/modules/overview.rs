use crate::imports::*;

pub struct Overview {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Overview {
    type Context = Core;

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // ui.label("Hello Overview");

        // return;

        #[allow(unused_mut)]
        let mut window = egui::Window::new("Hello World")
            .id(egui::Id::new("demo_window_options"));
         // required since we change the title
        // .resizable(resizable)
        // .constrain(constrain)
        // .collapsible(collapsible)
        // .title_bar(title_bar)
        // .scroll(scroll2)
        // .enabled(enabled);
        // if closable {
        //     window = window.open(open);
        // }
        // if anchored {
        //     window = window.anchor(anchor, anchor_offset);
        // }
        window.show(_ctx, |ui| {
            ui.label("Hello Overview");
        });


        egui::ScrollArea::vertical()
            .id_source("overview_scroll")
            .auto_shrink([false; 2])
            // .max_width(200.0)
            // .hscroll(false)
            // .stick_to_bottom(true)
            .show(ui, |ui| {

                core.nodes.iter().for_each(|(sid, nodes)| {

                    let machine_caption = if let Some(machine) = core.machines.get(sid) {
                        machine.get_caption()
                    } else {
                        format!("{:016x}", sid)
                    };

                    let machine_caption = RichText::new(machine_caption)
                        // .size(font_size)
                        // .family(FontFamily::Name(MNEMONIC_FONT.into()))
                        .family(FontFamily::Monospace)
                        .family(FontFamily::Name("noto_sans_mono_light".into()))
                        // .color(egui::Color32::WHITE)
                        ;

                    CollapsingHeader::new(machine_caption)
                        .default_open(true)
                        .id_source(sid)
                        .show(ui, |ui| {
            
                    // ui.collapsing(format!("{sid:016x}"), |ui| {
                        // ui.label(format!("System ID: {}", sid));
                            nodes.iter().for_each(|node| {

                                let node_caption = RichText::new(node.get_caption())
                                    // .size(font_size)
                                    .family(FontFamily::Name("noto_sans_mono_light".into()))
                                    // .family(FontFamily::Name(MNEMONIC_FONT.into()))
                                    .family(FontFamily::Monospace)
                                    // .color(egui::Color32::WHITE)
                                    ;
        

                                // ui.collapsing(format!("Node {uid:016x}"), |ui| {
                                    CollapsingHeader::new(node_caption)
                                        .default_open(false)
                                        .id_source(node.uid())
                                        .show(ui, |ui| {
                                                // ui.label(format!("Status: {:?}", node.status()));

                                            ui.set_max_width(400.0);
                                            ui.set_min_width(400.0);

                                            // ui.horizontal_wrapped(|ui|{

                                                match node.status() {
                                                    Status::Kaspa(status) => {

                                                        Metric::iter().chunks(5).into_iter().for_each(|chunk| {

                                                            ui.horizontal(|ui|{
                                                                chunk.into_iter().for_each(|metric| {
                                                                    let f = status.metrics_snapshot.get(metric);
                                                                    let text = metric.format(f,true,false);
                                                                    let (long,_short) = metric.title();
            
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(format!("{long}:"));
                                                                        ui.colored_label(egui::Color32::WHITE, text);
                                                                        ui.add_space(16.0); 
                                                                    });
                                                                });
                                                            });

                                                        });
                                                            
                                                    }
                                                    Status::Sparkle(_status) => {
                                                    }
                                                }

                                            // });

                                        });
                            });
                    });
                });
            });
    }
}
