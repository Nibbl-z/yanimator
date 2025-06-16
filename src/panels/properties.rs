use egui::{InputState, Key, Ui};

use crate::{anim_parser::{OAMFlip, OAMShape, OAMSize}, Yanimator};

fn get_size_string_with_shape<'a>(size: &'a OAMSize, shape: &'a OAMShape) -> &'a str {
    match shape {
        OAMShape::Square => {
            match size {
                OAMSize::Size0 => "8x8",
                OAMSize::Size1 => "16x16",
                OAMSize::Size2 => "32x32",
                OAMSize::Size3 => "64x64"
            }
        },
        OAMShape::Horizontal => {
            match size {
                OAMSize::Size0 => "16x8",
                OAMSize::Size1 => "32x8",
                OAMSize::Size2 => "32x16",
                OAMSize::Size3 => "64x32"
            }
        },
        OAMShape::Vertical => {
            match size {
                OAMSize::Size0 => "8x16",
                OAMSize::Size1 => "8x32",
                OAMSize::Size2 => "16x32",
                OAMSize::Size3 => "32x64"
            }
        }
    }
}

fn remove_oam(app: &mut Yanimator) {
    let cell  = match app.animation_cels.get_mut(&app.editing_cell) {
        Some(cell) => cell,
        None => return,
    };

    cell.oams.remove(app.editing_oam);
}

pub fn ui(ui: &mut Ui, app: &mut Yanimator) {
    ui.heading("Properties");
    
    let cell  = match app.animation_cels.get_mut(&app.editing_cell) {
        Some(cell) => cell,
        None => return,
    };
    
    let oam_len = cell.oams.len();
    
    let oam = match cell.oams.get_mut(app.editing_oam) {
        Some(oam) => oam,
        None => return,
    };
    
    //let sprites_len = app.spritesheet.sprites.len();
    //let palette_len = app.palette.palettes.len();

    egui::Grid::new("animation_cells")
        .num_columns(2)
        .striped(true)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
        
            ui.label("X");
            ui.add(egui::DragValue::new(&mut oam.x).speed(0.2));
            ui.end_row();

            ui.label("Y");
            ui.add(egui::DragValue::new(&mut oam.y).speed(0.2));
            ui.end_row();
        
            ui.label("Tile ID");
            ui.add(egui::DragValue::new(&mut oam.tile).speed(0.2).range(0..=app.spritesheet.sprites.len()));
            ui.end_row();
            
            ui.label("Palette ID");
            
            // the world would be a perfect place
            // if these drag value ranged didnt have to be inclusive
            
            let mut upper_range = app.palette.palettes.len();
            
            if upper_range > 0 {
                upper_range -= 1;
            }
            
            ui.add(egui::DragValue::new(&mut oam.palette).speed(0.2).range(0..=upper_range));
            ui.end_row();
            
            ui.label("Shape");
            egui::ComboBox::from_id_salt("shape_dropdown")
                .selected_text(format!("{:?}", &mut oam.shape))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut oam.shape, OAMShape::Square, "Square");
                    ui.selectable_value(&mut oam.shape, OAMShape::Horizontal, "Horizontal");
                    ui.selectable_value(&mut oam.shape, OAMShape::Vertical, "Vertical");
                });
            ui.end_row();

            ui.label("Size");
            egui::ComboBox::from_id_salt("size_dropdown")
                .selected_text(format!("{}", get_size_string_with_shape(&oam.size, &oam.shape)))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut oam.size, OAMSize::Size0, get_size_string_with_shape(&OAMSize::Size0, &oam.shape));
                    ui.selectable_value(&mut oam.size, OAMSize::Size1, get_size_string_with_shape(&OAMSize::Size1, &oam.shape));
                    ui.selectable_value(&mut oam.size, OAMSize::Size2, get_size_string_with_shape(&OAMSize::Size2, &oam.shape));
                    ui.selectable_value(&mut oam.size, OAMSize::Size3, get_size_string_with_shape(&OAMSize::Size3, &oam.shape));
                });
            ui.end_row();

            ui.label("Flip");
            egui::ComboBox::from_id_salt("flip_dropdown")
                .selected_text(format!("{:?}", &mut oam.flip))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut oam.flip, OAMFlip::None, "None");
                    ui.selectable_value(&mut oam.flip, OAMFlip::Horizontal, "Horizontal");
                    ui.selectable_value(&mut oam.flip, OAMFlip::Vertical, "Vertical");
                    ui.selectable_value(&mut oam.flip, OAMFlip::Both, "Both");
                });
            ui.end_row();
            
            ui.label("Z-Index");
            ui.add(egui::DragValue::new(&mut oam.zindex).speed(0.2).range(0..=oam_len));
            ui.end_row();
        });

    ui.separator();

    if ui.button("Delete OAM").clicked() {
        remove_oam(app);
    }
    
    let rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width().max(1.0), ui.available_height())
    );

    ui.allocate_rect(rect, egui::Sense::hover());
}

pub fn input(input: &InputState, app: &mut Yanimator) {
    let cell  = match app.animation_cels.get_mut(&app.editing_cell) {
        Some(cell) => cell,
        None => return,
    };
    
    let oam = match cell.oams.get_mut(app.editing_oam) {
        Some(oam) => oam,
        None => return,
    };

    if input.key_pressed(Key::ArrowUp) {
        oam.y -= 1;
    }

    if input.key_pressed(Key::ArrowDown) {
        oam.y += 1;
    }

    if input.key_pressed(Key::ArrowLeft) {
        oam.x -= 1;
    }

    if input.key_pressed(Key::ArrowRight) {
        oam.x += 1;
    }
}