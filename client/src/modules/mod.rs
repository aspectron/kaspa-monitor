use crate::imports::*;

pub type Module = workflow_egui::module::Module<Core>;

register_modules!(register_modules, [overview,]);
