use std::{collections::HashMap, fs, io::Write};

use serde::{Deserialize, Serialize};

use crate::{anim_parser::{Animation, AnimationCel, OAMFlip, OAMShape, OAMSize}, ProjectStructure};

/*
    
.yan format:

YAN file signature (3 bytes)
4 bytes for offset for animations

For each animation cell:

    Animation Cell Name
    0x00 seperator byte
    Amount of OAMs (1 byte)
    
    For each OAM:
        (Each OAM is 8 bytes)
        [yea, i could probably squish it back together into 6 bytes like how its actually stored]
        [but is it really worth it?? these files tend to be a few kilobytes at most]
        shape
        size
        flip
        x
        y
        palette
        tile (2 bytes, upper byte then lower byte)

For each animation:

    Animation name
    0x00 seperator byte
    Length of animation frames in bytes (2 bytes) (...hopefully an animation will never be more than 65535 bytes long..!)
    
    For each animation frame:
        Frame name
        0x00 seperator byte
        Frame duration (1 byte)
*/



pub fn create_project_bin(path: &str, animation_cells: &HashMap<String, AnimationCel>, animations: &Vec<Animation>) {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(String::from("YAN").as_bytes().to_vec());
    // We will set these bytes after putting in all the animation cells
    bytes.extend([0x00, 0x00, 0x00, 0x00]);

    for (name, cell) in animation_cells {
        // Cell name (saved as these may be editable in the future)
        let name_bytes = name.as_bytes().to_vec();
        bytes.extend(name_bytes);

        // Seperator byte
        bytes.push(0x00);

        // Cell length
        bytes.push(cell.oams.len() as u8);

        for oam in &cell.oams {
            bytes.push(oam.shape as u8);
            bytes.push(oam.size as u8);
            bytes.push(oam.flip as u8);
            bytes.push(oam.x as u8);
            bytes.push(oam.y as u8);
            bytes.push(oam.palette as u8);
            bytes.push((oam.tile >> 8) as u8);
            bytes.push((oam.tile & 0xFF) as u8);
        }
    }

    let animation_start_index: u32 = bytes.len() as u32;

    bytes[3] = (animation_start_index >> 24) as u8;
    bytes[4] = ((animation_start_index & 0xFF0000) >> 16) as u8;
    bytes[5] = ((animation_start_index & 0xFF00) >> 8) as u8;
    bytes[6] = (animation_start_index & 0xFF) as u8;

    for animation in animations {
        // Animation name
        let name_bytes = animation.name.as_bytes().to_vec();
        bytes.extend(name_bytes);

        // Seperator byte
        bytes.push(0x00);
        
        // Allocate 2 bytes for animation length
        bytes.extend([0x00, 0x00]);
        let start_length = bytes.len();

        for frame in &animation.frames {
            bytes.extend(frame.cell.as_bytes().to_vec());
            bytes.push(0x00);
            bytes.push(frame.duration);
        }

        let end_length = bytes.len();
        let length = (end_length - start_length) as u16;

        bytes[start_length - 2] = (length >> 8) as u8;
        bytes[start_length - 1] = (length & 0xFF) as u8;
    }

    let export = fs::File::create(path);
    
    if let Ok(mut file) = export {
        let _ = file.write_all(&bytes);
    }    
}

pub fn create_project_json(path: &str, animation_cells: &HashMap<String, AnimationCel>, animations: &Vec<Animation>) {
    let project = ProjectStructure {
        animation_cels: animation_cells.clone(),
        animations: animations.clone()
    };

    let export = fs::File::create(path);
    
    if let Ok(mut file) = export {
        if let Ok(bytes) = serde_json::to_vec_pretty(&project) {
            let _ = file.write_all(&bytes);
        }
    }   
}

pub fn export_animation_cels(path: &str, animation_cells: &HashMap<String, AnimationCel>) {
    let mut export = String::from("// Exported by Yanimator");

    for (name, cell) in animation_cells {
        export.push_str(&format!("\n\nAnimationCel {}[] = {{\n", name));
        export.push_str(&format!("    /* Len */ {},\n", cell.oams.len()));

        let mut i = 0;
        
        let mut sorted_oams = cell.oams.clone();
        sorted_oams.sort_by(|a, b| a.zindex.cmp(&b.zindex));
        
        for oam in sorted_oams {
            export.push_str(&format!("    /* {:0fill$} */ ", i, fill = 3));
            let shape: u16 = match oam.shape {
                OAMShape::Square => 0x0000,
                OAMShape::Horizontal => 0x4000,
                OAMShape::Vertical => 0x8000
            };

            let mut y = oam.y as i16;
            if y < 0 {
                y += 0x100;
            }

            let word1: u16 = shape | y as u16;

            export.push_str(&format!("0x{:0fill$x}, ", word1, fill = 4));

            let mut flip_size_nibble: u16 = match oam.size {
                OAMSize::Size0 => 0x0000,
                OAMSize::Size1 => 0x4000,
                OAMSize::Size2 => 0x8000,
                OAMSize::Size3 => 0xC000,
            };

            match oam.flip {
                OAMFlip::None => {},
                OAMFlip::Horizontal => {flip_size_nibble += 0x1000},
                OAMFlip::Vertical => {flip_size_nibble += 0x2000},
                OAMFlip::Both => {flip_size_nibble += 0x3000}
            };

            let mut x = oam.x as i16;
            if x < 0 {
                x += 0x200;
            }

            let word2: u16 = flip_size_nibble | x as u16;

            export.push_str(&format!("0x{:0fill$x}, ", word2, fill = 4));

            let palette = (oam.palette as u16) << 12;

            let tile = (oam.tile & 0x0FFF) as u16;

            let word3: u16 = palette | tile;


            export.push_str(&format!("0x{:0fill$x}", word3, fill = 4));

            i += 1;

            if i < cell.oams.len() {
                export.push_str(",\n");
            }
        }

        export.push_str("\n};");
    }

    let file = fs::File::create(path);
    
    if let Ok(mut file) = file {
        let _ = file.write_all(export.as_bytes());
    }
}

pub fn export_animations(path: &str, animations: &Vec<Animation>) {
    let mut export = String::from("// Exported by Yanimator");

    export.push_str("\n#include \"global.h\"\n#include \"graphics.h\"\n\n");
    export.push_str("// #include \"[Name of anim_cels.inc.c file goes here]\"\n\n");

    for animation in animations {
        export.push_str(&format!("struct Animation {}[] = {{\n", animation.name));

        let mut i = 0;
        for frame in &animation.frames {
            export.push_str(&format!("    /* {:0fill$} */ ", i, fill = 3));
            export.push_str(&format!("{{ {}, {} }},\n", frame.cell, frame.duration));
        
            i += 1;
        }

        export.push_str("    /* End */ END_ANIMATION\n};\n\n");
    }

    let file = fs::File::create(path);
    
    if let Ok(mut file) = file {
        let _ = file.write_all(export.as_bytes());
    }
}