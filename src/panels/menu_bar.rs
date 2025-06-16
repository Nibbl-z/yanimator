use std::{path::PathBuf};

use egui::{include_image, menu, Button, ColorImage, Key, KeyboardShortcut, Modifiers, TextureHandle, Ui};


use crate::{export, import, palette_parser::Palette, sprite_parser::Spritesheet, Yanimator};
use rfd::FileDialog;

const NEW_PROJECT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);
const OPEN_PROJECT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
const SAVE_PROJECT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);

fn open_project(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("Yanimator project", &["json"])
    .set_directory("/")
    .set_title("Select a Yanimator project")
    .pick_file() {
        Some(file) => file,
        None => return
    };

    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };
    
    if let Ok((animation_cels, animations)) = import::load_project_json(path_str) {
        app.animations = animations;
        app.animation_cels = animation_cels;
    }

    //let (animation_cels, animations) = import::load_project(path_str);
    
    
}

fn save_project(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("Yanimator project", &["json"])
    .set_directory("/")
    .set_title("Select save location")
    .save_file() {
        Some(file) => file,
        None => return
    };

    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };
    
    crate::export::create_project_json(path_str, &app.animation_cels, &app.animations);
}

fn load_palette(ui: &mut Ui, app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("Palette", &["pal"])
    .set_directory("/")
    .set_title("Select a palette")
    .pick_file() {
        Some(file) => file,
        None => return
    };
    
    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    app.palette = Palette::from_pal(path_str).unwrap();
    load_texture_handles(ui, app);
}

fn load_spritesheet(ui: &mut Ui, app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("4bpp spritesheet", &["4bpp"])
    .set_directory("/")
    .set_title("Select a spritesheet")
    .pick_file() {
        Some(file) => file,
        None => return
    };
    
    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    app.spritesheet = Spritesheet::from_4bpp(path_str).unwrap();
    load_texture_handles(ui, app);
}

fn load_texture_handles(ui: &mut Ui, app: &mut Yanimator) {
    let mut textures: Vec<Vec<TextureHandle>> = Vec::new();
    
    if app.palette.palettes.len() == 0 {return;}
    if app.spritesheet.sprites.len() == 0 {return;}
    
    for pal in app.palette.palettes.iter() {
        let mut palette_textures = Vec::new();

        for i in 0..app.spritesheet.sprites.len() {
            let sprite = &app.spritesheet.sprites[i];
            let mut pixels: Vec<u8> = Vec::new();
        
            for i in 0..0x40 {
                let palette_id = sprite.pixels[i];
                
                if palette_id == 0 {
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                } else {
                    let rgb = &pal[palette_id as usize];
                    pixels.push(rgb.r);
                    pixels.push(rgb.g);
                    pixels.push(rgb.b);
                    pixels.push(255);
                }
            }
            
            palette_textures.push(
                ui.ctx().load_texture(
                i.to_string(),
                ColorImage::from_rgba_unmultiplied([8, 8], &pixels), 
                egui::TextureOptions {
                    magnification: egui::TextureFilter::Nearest,
                    minification: egui::TextureFilter::Nearest,
                    wrap_mode: egui::TextureWrapMode::Repeat,
                    mipmap_mode: None,
                })
            )
        }

        textures.push(palette_textures);
    }

    app.textures = textures;
}

fn load_animation_cels(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("C", &["c"])
    .set_directory("/")
    .set_title("Select Animation Cels")
    .pick_file() {
        Some(file) => file,
        None => return
    };
    
    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    app.animation_cels = import::load_animation_cels_from_c(path_str);
}

fn load_animations(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("C", &["c"])
    .set_directory("/")
    .set_title("Select Animations")
    .pick_file() {
        Some(file) => file,
        None => return
    };
    
    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    app.animations = import::load_animations_from_c(path_str);
}

fn export_animation_cels(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("C", &["c"])
    .set_directory("/")
    .set_title("Select export location")
    .save_file() {
        Some(file) => file,
        None => return
    };

    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    export::export_animation_cels(path_str, &app.animation_cels);
}

fn export_animations(app: &mut Yanimator) {
    let file_path: PathBuf = match FileDialog::new()
    .add_filter("C", &["c"])
    .set_directory("/")
    .set_title("Select export location")
    .save_file() {
        Some(file) => file,
        None => return
    };

    let path_str = match file_path.to_str() {
        Some(path) => path,
        None => return
    };

    export::export_animations(path_str, &app.animations);
}

pub fn ui(ui: &mut Ui, app: &mut Yanimator) {
    if ui.input_mut(|i| i.consume_shortcut(&OPEN_PROJECT)) {
        open_project(app);
    }

    if ui.input_mut(|i| i.consume_shortcut(&SAVE_PROJECT)) {
        open_project(app);
    }

    menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.add(Button::image_and_text(include_image!("../../assets/page_add.png"), "New Project").shortcut_text(ui.ctx().format_shortcut(&NEW_PROJECT))).clicked() {
                // Blahh
            }
            
            if ui.add(Button::image_and_text(include_image!("../../assets/folder_page.png"), "Open Project").shortcut_text(ui.ctx().format_shortcut(&OPEN_PROJECT))).clicked() {
                open_project(app);
            }
            
            if ui.add(Button::image_and_text(include_image!("../../assets/page_save.png"), "Save Project").shortcut_text(ui.ctx().format_shortcut(&SAVE_PROJECT))).clicked() {
                save_project(app);
            }

            ui.separator();

            if ui.add(Button::image_and_text(include_image!("../../assets/picture_add.png"), "Load Spritesheet (.4bpp)")).clicked() {
                load_spritesheet(ui, app);
            }
            
            if ui.add(Button::image_and_text(include_image!("../../assets/palette.png"), "Load Palette (.pal)")).clicked() {
                load_palette(ui, app);
            }
            
            if ui.add(Button::image_and_text(include_image!("../../assets/film.png"), "Load Animation Cels (.c)")).clicked() {
                load_animation_cels(app);
            }

            if ui.add(Button::image_and_text(include_image!("../../assets/television.png"), "Load Animations (.c)")).clicked() {
                load_animations(app);
            }
            
            ui.separator();

            if ui.add(Button::image_and_text(include_image!("../../assets/film_save.png"), "Export Animation Cels (.c)")).clicked() {
                export_animation_cels(app);
            }

            if ui.add(Button::image_and_text(include_image!("../../assets/film_save.png"), "Export Animations (.c)")).clicked() {
                export_animations(app);
            }
        });
    });
}