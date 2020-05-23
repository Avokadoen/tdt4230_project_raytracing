use std::string::String;
use std::vec::Vec;

use super::pixel::Pixel;
use crate::resources::Resources;

// TODO: rename shader str builder
#[allow(dead_code)]
pub struct ShaderBuilder {
    shader_template: String,
    pixels: Vec<Pixel>,
}

impl ShaderBuilder {
    #[allow(dead_code)]
    pub fn new(name: &str, resources: &Resources) -> Result<Self, String> {
        let source = resources.load_cstring(name)
            .map_err(|e| format!("Error loading resource {}: {:?}", name, e))?;

        let shader_template = source.to_str().unwrap().to_owned(); // TODO: match case to get result

        if !is_valid_shader(&shader_template) {
            return Err(String::from("Invalid shader template"))
        }  

        Ok(Self {
            shader_template,
            pixels: Vec::default() 
        })
    }

    #[allow(dead_code)]
    pub fn append_pixel(&mut self, pixel: Pixel) -> &mut Self {
        self.pixels.push(pixel);
        self
    }

    #[allow(dead_code)]
    pub fn append_pixels(&mut self, pixels: &mut Vec<Pixel>) -> &mut Self {
        self.pixels.append(pixels);
        self
    }

    #[allow(dead_code)]
    pub fn build(&mut self) -> String {
        let mut shader_pixel_body = String::new();
        let mut first_if = true;
        for pixel in &self.pixels {
            let if_word = match first_if {
                true => "if",
                false => " else if",
            };
            first_if = false;

            let pixel_assigner = format!("{ifw} (current_color.w >= {id} && current_color.w <= {id}999) {sb}", ifw = if_word, id = pixel.alpha_id(), sb = "{");
            shader_pixel_body.push_str(pixel_assigner.as_str());
            shader_pixel_body.push_str(format!("{}{}", pixel.update_function(), "\n}").as_str());
        }

        self.shader_template.replace("// #TEMPLATE-PIXEL", shader_pixel_body.as_str())
    }
}

// TODO:
#[allow(dead_code)]
fn is_valid_shader(shader_template: &str) -> bool {
    true
}