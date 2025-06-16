use egui::{include_image, pos2, vec2, Button, Frame, Id, ImageButton, Modal, Rect, Scene, Sense, Ui};
use itertools::Itertools;

use crate::{anim_parser::AnimationCel, AppState, Yanimator};

pub struct AnimationCellsPanel {
    pub creation_modal_open: bool,
    deletion_confirmation_modal_open: bool,
    cell_name: String,
    deleting_cell: Option<String>
}

impl AnimationCellsPanel {
    pub fn init() -> Self {
        Self { 
            creation_modal_open: false,
            deletion_confirmation_modal_open: false,
            cell_name: String::from(""),
            deleting_cell: None
        }
    }
}

fn is_cell_name_invalid(app: &mut Yanimator) -> Option<String> {
    let cell_name = &app.animation_cells_panel.cell_name;
    
    if cell_name.len() == 0 {return None}
    if cell_name.contains(" ") {return Some(String::from("Cell name must not contain spaces"))}
    if !cell_name.chars().nth(0).unwrap().is_alphabetic() {return Some(String::from("First character in cell name must be a letter"))}
    if app.animation_cels.get(cell_name).is_some() {return Some(String::from("Cell name has already been used"))}

    None
}

fn create_animation_cell(app: &mut Yanimator) {
    app.animation_cells_panel.creation_modal_open = false;

    app.animation_cels.insert(app.animation_cells_panel.cell_name.clone(), AnimationCel {
        name: app.animation_cells_panel.cell_name.clone(),
        oams: Vec::new()
    });
}

fn remove_animation_cell(app: &mut Yanimator) {
    if let Some(deleting_cell) = &app.animation_cells_panel.deleting_cell {
        app.animation_cels.remove(deleting_cell);
        
        for animation in &mut app.animations {
            let mut removal_indexes: Vec<usize> = animation.frames.iter().enumerate().filter_map(|(_, frame)| {
                if frame.cell == *deleting_cell {
                    Some(frame.id)
                } else {None}
            }).collect();
            
            removal_indexes.reverse();

            for index in removal_indexes {
                animation.remove_anim_frame(index);
            }
        }
    }
}

pub fn ui(ui: &mut Ui, app: &mut Yanimator) {
    ui.horizontal(|ui| {
        ui.heading("Animation Cells");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_sized(vec2(20.0, 20.0), ImageButton::new(include_image!("../../assets/add.png"))).clicked() {
                app.animation_cells_panel.creation_modal_open = true;
                app.animation_cells_panel.cell_name = String::from("");
            }
        });
    });
    
    ui.separator();
    
    // i am not good enough at rust to deal with borrow checker issues like these,,,
    // i might try and fix this later but i've literally been banging my head on the wall 
    // for like 3 hours because of this

    /*let mut sorted_cels = Vec::new();
    

    let mut used_cels = Vec::new();
    {
        let animation = app.animations.get(app.animation_id);

        if let Some(animation) = animation {
            used_cels = animation.get_used_cels();
        }
    }
    
    sorted_cels.append(&mut used_cels);

    let animation_cels_keys: Vec<_> = app.animation_cels.keys().clone().sorted().collect();

    for cel in animation_cels_keys {
        if !sorted_cels.iter().any(|&c| c == cel) {
            sorted_cels.push(cel);
        }
    }*/
    
    egui::ScrollArea::vertical()
    .show(ui, |ui| {
        /*egui::Grid::new("animation_cells")
        .num_columns(1)
        .striped(true)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {*/
            for name in app.animation_cels.keys().sorted() {
                let cel = app.animation_cels.get(name).unwrap();

                ui.horizontal(|ui| {
                    let button = ui.add(Button::new(name).min_size(vec2(ui.available_width(), 20.0)));
                    
                    if button.double_clicked() {
                        app.state = AppState::CellEditor;
                        app.editing_cell = String::from(name);
                    }

                    button.context_menu(|ui| {
                        if ui.add(Button::image_and_text(include_image!("../../assets/film_edit.png"), "Edit")).clicked() {
                            app.state = AppState::CellEditor;
                            app.editing_cell = String::from(name);
                            ui.close_menu();
                        }

                        if ui.add(Button::image_and_text(include_image!("../../assets/delete.png"), "Delete")).clicked() {
                            app.animation_cells_panel.deleting_cell = Some(String::from(name));
                            app.animation_cells_panel.deletion_confirmation_modal_open = true;
                            ui.close_menu();
                        }

                        if ui.add(Button::image_and_text(include_image!("../../assets/keyframe_add.png"), "Insert Keyframe")).clicked() {
                            if let Some(animation) = app.animations.get_mut(app.animation_id) {
                                animation.insert_anim_frame(String::from(name), app.frames as isize);
                            }

                            ui.close_menu();
                        }
                    });

                    button.on_hover_ui_at_pointer(|ui| {    
                        ui.allocate_ui(vec2(100.0, 100.0), |ui| {
                            let mut rect = Rect::ZERO;                    
                            Scene::default()
                                .zoom_range(0.5..=0.5)
                                .show(ui, &mut rect, |ui| {
                                    cel.draw(&app.textures, ui, false);
                                });
                        });
                    });

                    /*ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_sized(vec2(20.0, 20.0), ImageButton::new(include_image!("../../assets/edit.png"))).clicked() {
                           
                        }
                        
                        if ui.add_sized(vec2(20.0, 20.0), ImageButton::new(include_image!("../../assets/keyframe_add.png"))).clicked() {
                            if let Some(animation) = app.animations.get_mut(app.animation_id) {
                                animation.insert_anim_frame(String::from(name), app.frames as isize);
                            }
                        }

                        if ui.add_sized(vec2(20.0, 20.0), ImageButton::new(include_image!("../../assets/delete.png"))).clicked() {
                            app.animation_cells_panel.deleting_cell = Some(String::from(name));
                            app.animation_cells_panel.deletion_confirmation_modal_open = true;
                        }
                    });*/
                });
            }
        //});
        
    });

    let rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width().max(1.0), ui.available_height())
    );

    if app.animation_cells_panel.creation_modal_open {
        Modal::new(Id::new("animation_cell_creation")).show(ui.ctx(), |ui| {
            ui.heading("Creating AnimationCel");
            ui.separator();
            
            ui.label("Cell Name:");
            let field = ui.text_edit_singleline(&mut app.animation_cells_panel.cell_name);
            
            match is_cell_name_invalid(app) {
                Some(message) => field.show_tooltip_text(message),
                None => {}
            }

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.animation_cells_panel.creation_modal_open = false;
                }

                if ui.button("Create").clicked() && app.animation_cells_panel.cell_name.len() > 0 && is_cell_name_invalid(app).is_none() {
                    create_animation_cell(app)
                }
            });
        });
    } else if app.animation_cells_panel.deletion_confirmation_modal_open {
        Modal::new(Id::new("animation_cell_deletion")).show(ui.ctx(), |ui| {
            if let Some(deleting_cell) = &app.animation_cells_panel.deleting_cell {
                ui.heading("Confirm Deletion");
                let seperator = ui.separator();
                
                ui.label(format!("Are you sure you want to delete {}?", deleting_cell));
                ui.label("This will also remove it from all animations.");
                
                ui.allocate_ui_with_layout(vec2(seperator.rect.width(), ui.available_height()), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Delete").clicked() {
                        remove_animation_cell(app);
                        app.animation_cells_panel.deletion_confirmation_modal_open = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        app.animation_cells_panel.deletion_confirmation_modal_open = false;
                    }
                });
            }
        });
    }
    
    ui.allocate_rect(rect, egui::Sense::hover());
}