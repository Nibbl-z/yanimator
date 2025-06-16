
use egui::{pos2, vec2, Color32, Rect, Stroke, TextureHandle, Ui};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum OAMShape {
    Square,
    Horizontal,
    Vertical
}
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum OAMSize {
    Size0,
    Size1,
    Size2,
    Size3
}
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum OAMFlip {
    None,
    Horizontal,
    Vertical,
    Both
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAM {
    pub shape: OAMShape,
    pub size: OAMSize,
    pub flip: OAMFlip,
    pub x: i8,
    pub y: i8,
    pub palette: usize,
    pub tile: usize,
    #[serde(skip)]
    pub selected: bool,
    #[serde(default = "usize::default")]
    pub zindex: usize
}

const SPRITE_SIZE: f32 = 20.0;

impl OAM {
    pub fn new(bytes: &[u8]) -> OAM {
        // 0xSYYY, 0xFXXX, 0xPTTT
        let word1 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
        let word2 = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
        let word3 = ((bytes[4] as u16) << 8) | (bytes[5] as u16);
        
        // TODO: probably throw a warning if shape/size are invalid
        
        let shape = match word1 >> 0xc {
            0x0 => OAMShape::Square,
            0x4 => OAMShape::Horizontal,
            0x8 => OAMShape::Vertical,
            _ => OAMShape::Square
        };

        let mut y= (word1 & 0x0FFF) as i16;
        if y >= 0x80 {
            y -= 0x100;
        }
        
        let flip_size_nibble = word2 >> 0xc;
        let size = match flip_size_nibble & !0x3 {
            0x0 => OAMSize::Size0,
            0x4 => OAMSize::Size1,
            0x8 => OAMSize::Size2,
            0xc => OAMSize::Size3,
            _ => OAMSize::Size0
        };

        let flip = match flip_size_nibble - (flip_size_nibble & !0x3) {
            0x0 => OAMFlip::None,
            0x1 => OAMFlip::Horizontal,
            0x2 => OAMFlip::Vertical,
            0x3 => OAMFlip::Both,
            _ => OAMFlip::None
        };
        
        let mut x = (word2 & 0x0FFF) as i16;

        if x >= 0x80 {
            x -= 0x200;
        }
        
        let palette = (word3 >> 0xc) as usize;
        let tile = (word3 & 0x0FFF) as usize;
        
        OAM {
            shape,
            size,
            flip,
            x: x as i8,
            y: y as i8,
            palette,
            tile,
            selected: false,
            zindex: 0
        }
    }

    pub fn from_bin(bytes: &[u8]) -> OAM {
        let shape = match bytes[0] {
            0 => OAMShape::Square,
            1 => OAMShape::Horizontal,
            2 => OAMShape::Vertical,
            _ => OAMShape::Square,
        };

        let size = match bytes[1] {
            0 => OAMSize::Size0,
            1 => OAMSize::Size1,
            2 => OAMSize::Size2,
            3 => OAMSize::Size3,
            _ => OAMSize::Size0,
        };

        let flip = match bytes[2] {
            0 => OAMFlip::None,
            1 => OAMFlip::Horizontal,
            2 => OAMFlip::Vertical,
            3 => OAMFlip::Both,
            _ => OAMFlip::None
        };

        let x = bytes[3] as i8;
        let y = bytes[4] as i8;
        let palette = bytes[5] as usize;
        let tile = (((bytes[6] as usize) << 8) | (bytes[7] as usize)) as usize;

        OAM {shape, size, flip, x, y, palette, tile, selected: false, zindex: 0}
    }
    
    pub fn get_width_and_height(&self) -> (usize, usize) {
        match self.shape {
            OAMShape::Square => match self.size {
                OAMSize::Size0 => (1, 1),
                OAMSize::Size1 => (2, 2),
                OAMSize::Size2 => (4, 4),
                OAMSize::Size3 => (8, 8),
            },
            OAMShape::Horizontal => match self.size {
                OAMSize::Size0 => (2, 1),
                OAMSize::Size1 => (4, 1),
                OAMSize::Size2 => (4, 2),
                OAMSize::Size3 => (8, 4),
            },
            OAMShape::Vertical => match self.size {
                OAMSize::Size0 => (1, 2),
                OAMSize::Size1 => (1, 4),
                OAMSize::Size2 => (2, 4),
                OAMSize::Size3 => (4, 8),
            }
        }
    }

    pub fn get_sprite_indexes(&self) -> Vec<Vec<usize>> {
        let mut sprite_indexes: Vec<Vec<usize>> = Vec::new();
        
        let (width, height) = self.get_width_and_height();
        
        let mut y_range: Vec<usize> = (0..height).collect();
        let mut x_range: Vec<usize> = (0..width).collect();
        
        match self.flip {
            OAMFlip::Horizontal => {
                x_range = (0..width).rev().collect();
            },
            OAMFlip::Vertical => {
                y_range = (0..height).rev().collect();
            },
            OAMFlip::Both => {
                x_range = (0..width).rev().collect();
                y_range = (0..height).rev().collect();
            },
            OAMFlip::None => {}
        }

        for y in y_range {
            let mut row: Vec<usize> = Vec::new();
            
            for &x in &x_range {
                row.push(self.tile + x + y * 32);
            }

            sprite_indexes.push(row);
        }
        
        return sprite_indexes;
    }

    pub fn get_sprite_indexes_one_dimensional(&self) -> Vec<usize> {
        let two_dimensional_indexes = self.get_sprite_indexes();
        let mut indexes = Vec::new();

        for y in two_dimensional_indexes {
            for x in y {
                indexes.push(x);
            }
        }

        return indexes;
    }

    pub fn draw(&self, textures: &Vec<Vec<TextureHandle>>, ui: &mut Ui, selection_indicator_enabled: bool) {
        let oam_sprites = self.get_sprite_indexes();
            
        
        let (width, height) = self.get_width_and_height();

        for y in 0..height {
            for x in 0..width {
                
                let texture_sheet = match textures.get(self.palette) {
                    Some(texture) => texture,
                    None => continue
                };

                if oam_sprites[y][x] >= texture_sheet.len() {continue;}
                
                let rect = egui::Rect::from_min_size(
                    pos2(
                        (x as f32) * SPRITE_SIZE + (self.x as f32) * SPRITE_SIZE / 8.0, 
                        (y as f32) * SPRITE_SIZE + (self.y as f32) * SPRITE_SIZE / 8.0),
                    vec2(SPRITE_SIZE, SPRITE_SIZE)
                );
                
                let source = match texture_sheet.get(oam_sprites[y][x]) {
                    Some(source) => source,
                    None => continue
                };

                ui.put(rect, |ui: &mut Ui| {
                    
                    let mut texture = egui::Image::new(source);
                    
                    match self.flip { 
                        OAMFlip::Horizontal => {
                            texture = texture.uv(Rect::from_min_max(pos2(1.0, 0.0), pos2(0.0, 1.0)));
                        },
                        OAMFlip::Vertical => {
                            texture = texture.uv(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)));
                        },
                        OAMFlip::Both => {
                            texture = texture.uv(Rect::from_min_max(pos2(1.0, 1.0), pos2(0.0, 0.0)));
                        },
                        _ => {}
                    }

                    if self.selected && selection_indicator_enabled {
                        
                        texture = texture.tint(Color32::LIGHT_GREEN);
                    }
                    
                    ui.add(
                        texture.fit_to_exact_size(vec2(SPRITE_SIZE, SPRITE_SIZE))
                    )
                });
                
                //ui.allocate_space(vec2(SPRITE_SIZE, SPRITE_SIZE));
            }
        }
        
        
        
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimationCel {
    pub name: String,
    pub oams: Vec<OAM>
}

fn parse_hex_string(string: &str) -> Option<u8> {
    match u8::from_str_radix(&string, 16) {
        Ok(value) => Some(value),
        Err(_) => None
    }
}

impl AnimationCel {
    pub fn from_c(c: &str, name: &str) -> Option<AnimationCel> {
        let oam_regex = Regex::new(r"0x[0-9a-fA-F]{4}").unwrap();

        let words: Vec<_> = oam_regex.find_iter(c).map(|m| m.as_str()).collect();
        let mut i = 0;

        let mut oams: Vec<OAM> = Vec::new();
        let mut zindex = 0;

        while i < words.len() {
            let mut bytes: Vec<u8> = Vec::new();

            let word1 = words.get(i)?;
            let word2 = words.get(i + 1)?;
            let word3 = words.get(i + 2)?;

            let byte1 = parse_hex_string(&word1[2..4])?;
            let byte2 = parse_hex_string(&word1[4..6])?;
            let byte3 = parse_hex_string(&word2[2..4])?;
            let byte4 = parse_hex_string(&word2[4..6])?;
            let byte5 = parse_hex_string(&word3[2..4])?;
            let byte6 = parse_hex_string(&word3[4..6])?;
            
            bytes.push(byte1);
            bytes.push(byte2);
            bytes.push(byte3);
            bytes.push(byte4);
            bytes.push(byte5);
            bytes.push(byte6);
            
            i += 3;

            let mut oam = OAM::new(&bytes);
            oam.zindex = zindex;

            oams.push(oam);
            
            zindex += 1;
        }

        Some(AnimationCel { oams, name: name.to_string() })
    }

    pub fn from_bin(bin: &[u8]) -> Option<AnimationCel> {
        let mut name = String::from("");
        let mut i = 0;

        while bin[i] != 0x00 {
            name.push(bin[i] as char);
            i += 1;
        }

        i += 1;

        let length = bin[i] as usize;
        let mut oams = Vec::new();
        i += 1;
        for x in 0..length {
            oams.push(OAM::from_bin(&bin[i + (x * 8)..i + (x * 8) + 8]))
        }
        

        Some(AnimationCel { name, oams })
    }

    pub fn draw(&self, textures: &Vec<Vec<TextureHandle>>, ui: &mut Ui, selection_indicator_enabled: bool) {
        let mut sorted_oams = self.oams.clone();
        sorted_oams.sort_by(|a, b| a.zindex.cmp(&b.zindex));
        
        let mut selected_oam = None;

        for oam in sorted_oams.iter().rev() {
            oam.draw(textures, ui, selection_indicator_enabled);
            if oam.selected {
                selected_oam = Some(oam);
            }
        }
        
        if let Some(selected_oam) = selected_oam {
            if selection_indicator_enabled {
                let (width, height) = selected_oam.get_width_and_height();
                ui.painter().rect_stroke(
                    Rect::from_min_size(pos2((selected_oam.x as f32) * SPRITE_SIZE / 8.0, (selected_oam.y as f32) * SPRITE_SIZE / 8.0), vec2(SPRITE_SIZE * width as f32, SPRITE_SIZE * height as f32)), 
                    0, 
                    Stroke::new(2.0, Color32::RED), 
                    egui::StrokeKind::Outside
                );
            }
        }

        
    }
}


#[derive(Deserialize, Serialize, Clone)]
pub struct AnimationFrame {
    pub cell: String,
    pub duration: u8,
    #[serde(skip)]
    pub id: usize
}

pub struct PositionedAnimationFrame {
    cell: String,
    pub position: isize,
    id: usize
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub name: String,
    #[serde(skip)]
    pub current_frame: usize,
    #[serde(skip)]
    pub duration: usize
}

impl Animation {
    pub fn from_c(c: &str, name: &str) -> Option<Animation> {
        let mut frame_positions = Vec::new();
        let mut i = 0;
        let mut total_duration = 0;

        while let Some(pos) = c[i..].find("{") {
            frame_positions.push(i + pos);
            i += pos + 4;
        }
        frame_positions.remove(0);

        let mut frames = Vec::new();

        for pos in frame_positions.into_iter() {
            let mut cel_name = String::new();
            let mut duration_str = String::new();

            i = pos + 1;
            while c.chars().nth(i) != Some(',') {
                if c.chars().nth(i) != Some(' ') {
                    cel_name.push(c.chars().nth(i).unwrap());
                }
                i += 1;
            }

            i += 1;

            while c.chars().nth(i) != Some('}') {
                if c.chars().nth(i) != Some(' ') {
                    duration_str.push(c.chars().nth(i).unwrap());
                }
                i += 1;
            }

            let duration = match duration_str.parse() {
                Ok(value) => value,
                Err(_) => return None,
            };

            frames.push(AnimationFrame {
                cell: cel_name,
                duration,
                id: frames.len()
            });
            
            total_duration += duration as usize;
        }

        Some(Animation { frames, name: name.to_string(), current_frame: 0, duration: total_duration })
    }

    pub fn from_bin(bin: &[u8]) -> Option<Animation> {
        let mut name = String::from("");
        let mut i = 0;
        let mut frame_id = 0;
        let mut duration = 0;

        while bin[i] != 0x00 {
            name.push(bin[i] as char);
            i += 1;
        }
        
        // Skip over animation length
        i += 3;

        let mut frames = Vec::new();
        let mut cell = String::from("");
        while i < bin.len() {
            
            if bin[i] != 0x00 {
                cell.push(bin[i] as char);
            } else {
                i += 1; // Go to duration byte
                frames.push(AnimationFrame {
                    cell,
                    duration: bin[i],
                    id: frame_id
                });
                duration += bin[i] as usize;
                frame_id += 1;
                cell = String::from("");
            }
            
            i += 1;
            
        }

        Some(Animation { frames, name, current_frame: 0, duration })
    }

    /*pub fn get_total_frame_duration(&self, index: usize) -> usize {
        let mut result = 0;
        
        for i in 0..index {
            result += self.frames[i].duration as usize;
        }

        result
    }*/

    pub fn get_total_frames(&self) -> usize {
        let mut result = 0;

        for frame in &self.frames {
            result += frame.duration as usize;
        }
        
        result
    }
    
    pub fn get_anim_frame_from_frames(&self, frames: usize) -> usize {
        if frames > self.get_total_frames() {
            return 0;
        }

        if self.frames.len() == 0 {
            return 0;
        }

        let mut result = 0;
        let mut i = 0;
        let mut current_frame = &self.frames[0];
        
        for _ in 0..frames {
            if i == current_frame.duration {
                i = 0;
                result += 1;
                current_frame = &self.frames[result];
            }
            i += 1;
        }

        result
    }

    pub fn convert_duration_frames_to_positioned(frames: &Vec<AnimationFrame>) -> Vec<PositionedAnimationFrame> {
        let mut positioned_frames = Vec::new();
        let mut total_duration = 0;
        
        for frame in frames {
            positioned_frames.push(PositionedAnimationFrame {
                cell: frame.cell.clone(),
                position: total_duration,
                id: frame.id
            });

            total_duration += frame.duration as isize;
        }
        
        positioned_frames
    }

    pub fn convert_positioned_frames_to_duration(mut frames: Vec<PositionedAnimationFrame>, duration: usize) -> Vec<AnimationFrame> {
        frames.sort_by(|a, b| a.position.cmp(&b.position));
        
        let mut duration_frames = Vec::new();
        
        if frames.len() == 0 {
            return duration_frames;
        }

        for i in 0..frames.len() {
            let frame = &frames[i];
            let next_frame_pos = if i == frames.len() - 1 {
                duration as isize
            } else {
                frames[i + 1].position
            };

            duration_frames.push(AnimationFrame { 
                cell: frame.cell.clone(), 
                duration: (next_frame_pos - frame.position) as u8,
                id: frame.id
            });
        }

        duration_frames
    }
    
    pub fn move_anim_frame(&mut self, frame_id: usize, offset: isize) -> Option<()> {
        if frame_id == 0 {return None}
        if offset == 0 {return None}
        
        let mut positioned_frames = Animation::convert_duration_frames_to_positioned(&self.frames);
        let frame_to_edit = positioned_frames.iter_mut().find(|k| k.id == frame_id)?;
        
        frame_to_edit.position += offset;
        self.frames = Animation::convert_positioned_frames_to_duration(positioned_frames, self.duration);
        
        Some(())
    }
    
    pub fn insert_anim_frame(&mut self, cell: String, position: isize) {
        let mut positioned_frames = Animation::convert_duration_frames_to_positioned(&self.frames);
        
        positioned_frames.push(PositionedAnimationFrame { cell, position, id: positioned_frames.len() + 1 });
        
        self.frames = Animation::convert_positioned_frames_to_duration(positioned_frames, self.duration);
    }
    
    pub fn remove_anim_frame(&mut self, frame_id: usize) {
        if frame_id == 0 {return;}
        
        if let Some(index) = self.frames.iter().position(|f| f.id == frame_id) {
            let duration = self.frames[index].duration;
            self.frames.remove(index);
            
            self.frames[index - 1].duration += duration;
        }
    }

    pub fn get_minimum_duration(&self) -> usize {
        let positioned_frames = Animation::convert_duration_frames_to_positioned(&self.frames);
            
        if positioned_frames.len() == 0 {
            return 0;
        }

        if let Some(last_frame) = positioned_frames.get(positioned_frames.len() - 1) {
            last_frame.position as usize
        } else {
            0
        }
    }

    pub fn update_duration(&mut self) {
        let minimum_duration = self.get_minimum_duration();
        if let Some(frame) = self.frames.last_mut() {
            frame.duration = (self.duration - minimum_duration) as u8;
        }
    }
    pub fn get_used_cels(&self) -> Vec<&String> {
        let mut used_cels = Vec::new();

        for frame in &self.frames {
            if !used_cels.iter().any(|&cel| cel == &frame.cell) {
                used_cels.push(&frame.cell);
            }
        }

        used_cels
    }
}