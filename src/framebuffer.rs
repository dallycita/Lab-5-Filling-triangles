use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    // 4 valores por píxel: rojo, verde, azul y alfa (0 a 255).
    pixels: Vec<u8>,
    background_color: Vector3,
    texture: Option<Texture2D>,
}

/// Convierte un color con componentes entre 0.0 y 1.0 a valores entre 0 y 255.
fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Framebuffer {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
            background_color: Vector3::new(0.0, 0.0, 0.0),
            texture: None,
        }
    }

    pub fn set_background_color(&mut self, color: Vector3) {
        self.background_color = color;
    }

    pub fn clear(&mut self) {
        let r = to_byte(self.background_color.x);
        let g = to_byte(self.background_color.y);
        let b = to_byte(self.background_color.z);
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
            pixel[3] = 255;
        }
    }

    /// Escribe un color en el framebuffer. Las posiciones fuera de la imagen se ignoran.
    pub fn point(&mut self, x: i32, y: i32, color: Vector3) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let index = ((y * self.width + x) * 4) as usize;
        self.pixels[index] = to_byte(color.x);
        self.pixels[index + 1] = to_byte(color.y);
        self.pixels[index + 2] = to_byte(color.z);
        self.pixels[index + 3] = 255;
    }

    /// Crea la textura donde se copia el framebuffer para mostrarlo en la ventana.
    pub fn init_texture(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        let image = Image::gen_image_color(self.width, self.height, Color::BLACK);
        let texture = window
            .load_texture_from_image(thread, &image)
            .expect("No se pudo crear la textura");
        self.texture = Some(texture);
    }

    /// Copia el framebuffer a la textura y la dibuja en la ventana.
    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        if let Some(texture) = &mut self.texture {
            texture
                .update_texture(&self.pixels)
                .expect("No se pudo actualizar la textura");

            let mut d = window.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            d.draw_texture(&*texture, 0, 0, Color::WHITE);
        }
    }

    /// Guarda el contenido del framebuffer como imagen PNG.
    pub fn render_to_file(&self, path: &str) {
        let mut image = Image::gen_image_color(self.width, self.height, Color::BLACK);
        for y in 0..self.height {
            for x in 0..self.width {
                let i = ((y * self.width + x) * 4) as usize;
                let color = Color::new(
                    self.pixels[i],
                    self.pixels[i + 1],
                    self.pixels[i + 2],
                    255,
                );
                image.draw_pixel(x, y, color);
            }
        }
        image.export_image(path);
    }
}
