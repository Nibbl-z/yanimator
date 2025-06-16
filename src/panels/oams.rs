use egui::{include_image, vec2, ImageButton, Ui};

use crate::{anim_parser::{OAMFlip, OAMShape, OAMSize, OAM}, Yanimator};

pub fn ui(ui: &mut Ui, app: &mut Yanimator) {
    let cell = match app.animation_cels.get_mut(&app.editing_cell) {
        Some(cell) => cell,
        None => return,
    };

    ui.horizontal(|ui| {
        ui.heading("OAMs");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_sized(vec2(20.0, 20.0), ImageButton::new(include_image!("../../assets/add.png"))).clicked() {
                cell.oams.push(OAM {
                    shape: OAMShape::Square,
                    size: OAMSize::Size0,
                    flip: OAMFlip::None,
                    x: 0,
                    y: 0,
                    palette: 0,
                    tile: 0,
                    selected: false,
                    zindex: cell.oams.len()
                });
            }
        });
    });
    
    egui::ScrollArea::vertical()
    .show(ui, |ui| {
        egui::Grid::new("oams")
        .num_columns(1)
        .striped(true)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            let mut i = 0;
            
            for _oam in &cell.oams {
                if ui.button(format!("OAM_{}", i)).clicked() {
                    app.editing_oam = i;
                }
                
                i += 1;
                ui.end_row();
            }
        });
    });
    
    let rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width().max(1.0), ui.available_height())
    );
    ui.allocate_rect(rect, egui::Sense::hover());
}