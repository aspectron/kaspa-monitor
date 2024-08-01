// use std::collections::btree_map;//::Entry;
use std::collections::hash_map; //::Entry;

use crate::imports::*;

pub struct Core {
    pub manager: ModuleManager<Self>,
    // pub nodes : AHashMap<u64,AHashMap<u64,Node>>,
    pub nodes: AHashMap<u64, Vec<Node>>,
    // pub nodes : BTreeMap<u64,BTreeMap<u64,Node>>,
    pub machines: AHashMap<u64, Machine>,
}

impl Core {
    /// Core initialization
    pub fn try_new(cc: &eframe::CreationContext<'_>, runtime: Runtime) -> Result<Self> {
        // crate::fonts::init_fonts(cc);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = FontDefinitions::default();
        fonts.add_static(
            // FontFamily::Monospace,
            FontFamily::Name("noto_sans_mono_light".into()),
            "noto_sans_mono_light",
            include_bytes!("../resources/fonts/NotoSansMono-Light.ttf"),
        );
        fonts.add_static(
            FontFamily::Monospace,
            "ubuntu_mono",
            include_bytes!("../resources/fonts/UbuntuMono-Regular.ttf"),
        );
        cc.egui_ctx.set_fonts(fonts);

        runtime.bind(Arc::new(crate::services::MonitorService::default()));

        let modules = crate::modules::register_modules(&runtime)
            .into_iter()
            .collect();

        let manager = ModuleManager::new(TypeId::of::<modules::Overview>(), modules);

        Ok(Self {
            manager,
            nodes: Default::default(),
            machines: Default::default(),
        })
    }

    pub fn handle_update(&mut self, update: &Update) {
        match update {
            Update::Status { status } => {
                let sid = status.sid();
                let uid = status.uid();

                let nodes = self.nodes.entry(sid).or_default();

                // let node = nodes.iter_mut().find(|node| node.uid() == uid);
                if let Some(node) = nodes.iter_mut().find(|node| node.uid() == uid) {
                    node.set_status(status.clone());
                } else {
                    nodes.push(Node::new(status.clone()));
                    nodes.sort_by_key(|node| node.network_id());
                }

                // match nodes_by_sid.entry(uid) {
                //     hash_map::Entry::Occupied(mut node) => {
                //         node.get_mut().set_status(status.clone());
                //     }
                //     hash_map::Entry::Vacant(node) => {
                //         node.insert(Node::new(status.clone()));
                //     }
                // }
                // node.set_status(status.clone());

                // let node = self.nodes.entry(*node_id).or_insert_with(AHashMap::new);
                // node.insert(status.id, status.clone());
            }
            Update::Caps { uid: _, caps } => {
                let sid = caps.system_id();

                match self.machines.entry(sid) {
                    hash_map::Entry::Occupied(mut machine) => {
                        machine.get_mut().set_caps(caps.clone());
                    }
                    hash_map::Entry::Vacant(machine) => {
                        machine.insert(Machine::new(caps.clone()));
                    }
                }
                // let node = nodes_by_sid.entry(*uid).or_default();
                // machine.set_caps(caps.clone());
            }
        }
    }
}

impl App for Core {
    #[cfg(not(target_arch = "wasm32"))]
    fn on_exit(&mut self) {
        // self.is_shutdown_pending = true;
        Runtime::halt();
        println!("bye!");
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.manager.clone().render(self, ctx, _frame, ui);
        });
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     self.manager.clone().render(self, ctx, _frame, ui);
        // });
    }

    fn handle_event(&mut self, _ctx: &egui::Context, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Application(application_event) => {
                match application_event.as_ref::<Event>() {
                    Event::Update { update } => {
                        self.handle_update(update);
                    }
                }
            }
            RuntimeEvent::Exit => {
                cfg_if! {
                    if #[cfg(not(target_arch = "wasm32"))] {
                        // self.is_shutdown_pending = true;
                        _ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
            v => {
                println!("Unhandled event: {:?}", v);
            }
        }
    }
}

impl Core {}
