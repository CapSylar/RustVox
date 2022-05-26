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
        ui.text_wrapped("Colors");
        ui.separator();
        ui.slider("Red", 0.0, 1.0, &mut state.red);
        ui.slider("Green", 0.0, 1.0, &mut state.green);
        ui.slider("Blue", 0.0, 1.0, &mut state.blue);
        ui.separator();
        ui.text("position settings");
        ui.separator();
        ui.slider("rotation in x" , 0.0 , PI , &mut state.rotation_x);
        ui.slider("rotation in z" , 0.0 , PI , &mut state.rotation_z);
        ui.slider("depth in z", 0.0 , -50.0 ,  &mut state.depth_z);
        ui.slider("view x" , -5.0 , 5.0 , &mut state.view_x);
        ui.slider("view y", -5.0 , 5.0 , &mut state.view_y);
        ui.separator();
        ui.text_wrapped("Euler Angles");
        ui.text(format!("Pitch: {}", state.pitch));
        ui.text(format!("Yaw: {}", state.yaw));
    
        ui.text(format!("frame time: {}us" , state.frame_time));
        ui.text(format!("FPS: {}", 1.0/(state.frame_time as f32 / 1000000.0) ));
    
        font.pop();});
    }
}

