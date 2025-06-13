use std::{collections::HashMap, fs};

use crate::{anim_parser::{Animation, AnimationCel}, ProjectStructure};

pub fn load_project(path_str: &str) -> (HashMap<String, AnimationCel>, Vec<Animation>) {
    let project_bytes = fs::read(path_str).unwrap();

    let mut i;

    let mut animation_cels: HashMap<String, AnimationCel> = HashMap::new();
    
    
    let mut current_start_index = 7;
    let mut current_end_index;
    let mut name_length = 0;
    let mut read_name = false;
    
    i = 7;
    
    let animations_offset: u32 = ((project_bytes[3] as u32) << 24) | ((project_bytes[4] as u32) << 16) | ((project_bytes[5] as u32) << 8) | project_bytes[6] as u32;
    
    while i < animations_offset as usize {
        let byte = project_bytes[i];
        
        if byte == 0x00 && !read_name {
            read_name = true;
        }

        i += 1;
        name_length += 1;
        // We are now on the cell length byte
        if read_name {
            current_end_index = current_start_index + name_length + project_bytes[i] as usize * 8 + 1;
            
            let cell = AnimationCel::from_bin(&project_bytes[current_start_index..current_end_index]);
            if let Some(cell) = cell {
                animation_cels.insert(cell.name.clone(), cell);
            }
            
            read_name = false;
    
            name_length = 0;
            i = current_end_index;
            current_start_index = i;
        }
    }
    
    // Load Animations
    
    let mut animations = Vec::new();
    
    while i < project_bytes.len() {
        let byte = project_bytes[i];
        
        if byte == 0x00 && !read_name {
            read_name = true;
        }
        
        i += 1;
        name_length += 1;
    
        // We are now on the animation byte length

        if read_name {
            let upper_byte = project_bytes[i];
            i += 1;
            let lower_byte = project_bytes[i];
            let animation_length = (((upper_byte as u16) << 8) | lower_byte as u16) as usize;
            
            current_end_index = current_start_index + name_length + animation_length + 2;
            
            let animation = Animation::from_bin(&project_bytes[current_start_index..current_end_index]);
            
            if let Some(animation) = animation {
                animations.push(animation);
            }
            
            read_name = false;
            
            name_length = 0;
            i = current_end_index;
            current_start_index = i;
        }
    }

    (animation_cels, animations)
}

pub fn load_project_json(path_str: &str) -> Result<(HashMap<String, AnimationCel>, Vec<Animation>), serde_json::Error> {
    let project_str = fs::read(path_str).unwrap();
    let mut project: ProjectStructure = serde_json::from_slice(&project_str)?;

    for animation in &mut project.animations {
        animation.duration = animation.get_total_frames();
        
        let mut i = 0;
        for frame in &mut animation.frames {
            frame.id = i;
            i += 1;
        }
    }

    Ok((project.animation_cels, project.animations))
}

pub fn load_animation_cels_from_c(path_str: &str) -> HashMap<String, AnimationCel> {
    let mut i = 0;
    let cels_file = fs::read_to_string(path_str).unwrap();

    let mut cel_positions = Vec::new();


    while let Some(pos) = cels_file[i..].find("AnimationCel ") {
        cel_positions.push(i + pos + 13);
        i += pos + 7;
    }

    cel_positions
        .iter()
        .filter_map(|&start| {
            let sliced_cel = &cels_file[start..];
            let cel_name_end = sliced_cel.find('[')?;
            let cel_name = &sliced_cel[..cel_name_end];

            let cel_str_start = cel_name_end + 1;
            let cel_str_end = sliced_cel[cel_str_start..].find(';')?;
            let cel_str = &sliced_cel[cel_str_start..cel_str_start + cel_str_end];

            AnimationCel::from_c(cel_str, cel_name)
        })
        .map(|cel| (cel.name.clone(), cel))
        .collect()
}

pub fn load_animations_from_c(path_str: &str) -> Vec<Animation> {
    let anim_file = fs::read_to_string(path_str).unwrap();

    let mut anim_positions = Vec::new();
    let mut i = 0;

    while let Some(pos) = anim_file[i..].find("struct Animation ") {
        anim_positions.push(i + pos + 17);
        i += pos + 17;
    }
    
    anim_positions
        .iter()
        .filter_map(|&start| {
            let sliced_anim = &anim_file[start..];
            let anim_name_end = sliced_anim.find('[')?;
            let anim_name = &sliced_anim[..anim_name_end];

            let anim_str_start = anim_name_end + 1;
            let anim_str_end = sliced_anim[anim_str_start..].find(';')?;
            let anim_str = &sliced_anim[anim_str_start..anim_str_start + anim_str_end];
            
            Animation::from_c(&anim_str, &anim_name)
        })
        .collect()
}