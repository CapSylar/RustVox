use std::{f32::consts::PI};
use imgui::{Condition, FontSource, Context, FontId};
use crate::State;

pub struct Ui
{
    pub used_font: FontId,
}

impl Ui
{
    pub fn new(imgui: &mut Context) -> Ui
    {
        let used_font = imgui.fonts().add_font(&[FontSource::TtfData{
            data: include_bytes!("../resources/JetBrains.ttf"),
            size_pixels: 17.0 ,
            config: None,
        }]);

        Ui{ used_font }
    }
    
    //FIXME: fix warning 
    pub fn build_ui(&self, ui: & imgui::Ui , state:&mut State)
    {
        let font = ui.push_font(self.used_font);
        ui.window("Tab")
        .position([0.0,0.0], Condition::Always)
        .size([400.0, 600.0], Condition::FirstUseEver)
        .build(|| {
        ui.text_wrapped("Controls");
        ui.separator();
        ui.text("Num1 to Toggle Mouse");
        ui.text("Num2 to Toggle Line Mode");
        ui.text("NUm3 to Toggle between Vsync Off/On");
        ui.separator();
        ui.text_wrapped("Performance");
        ui.text(format!("frame time: {}us" , state.frame_time));
        ui.text(format!("FPS: {}", 1.0/(state.frame_time as f32 / 1000000.0) ));
        font.pop();});
    }
}

