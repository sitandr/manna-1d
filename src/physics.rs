use egui::{Color32, ColorImage, Image, Sense, TextureHandle, TextureOptions, Vec2};
// use rand::Rng;
//  use rand_distr::{StandardNormal};
use rand_core::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy)]
pub enum Modification {
    #[default]
    Ignored,
    Increased,
    Decreased,
}

impl Modification {
    pub fn get_color(&self) -> Color32 {
        match self {
            Modification::Ignored => Color32::WHITE,
            Modification::Increased => Color32::RED,
            Modification::Decreased => Color32::BLUE,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Simulation1D {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<(u8, Modification)>,
    #[serde(skip)]
    rng: RngWrapper,

    #[serde(skip)]
    texture: Option<TextureHandle>,
    zoom: f32,
    center: f32,

    pub fixed_point: bool,
}

impl Default for Simulation1D {
    fn default() -> Self {
        Self {
            width: 100,
            height: 8,
            cells: vec![(0, Modification::Ignored); 800],
            rng: Default::default(),
            texture: Default::default(),
            zoom: 1.,
            center: 50.,
            fixed_point: false,
        }
    }
}

impl Simulation1D {
    // if true, need to change the state
    pub fn random_step(&mut self) -> bool {
        let i = if self.fixed_point {
            self.width / 2
        } else {
            self.rng.0.next_u32() as usize % self.width
        };
        for c in &mut self.cells {
            c.1 = Modification::Ignored;
        }
        self.cells[i].0 += 1;
        self.cells[i].1 = Modification::Increased;

        self.cells[i].0 > 2
    }

    // if true, need to change the state
    pub fn step(&mut self) -> u32 {
        let mut avalanche_size = 0;
        for c in &mut self.cells {
            c.1 = Modification::Ignored;
        }

        for i in 0..self.width {
            if self.cells[i].0 > 2 {
                let x = (self.rng.0.next_u32() % 2) as u8;
                let y = (self.rng.0.next_u32() % 2) as u8;

                self.cells[i].0 -= 2;
                self.cells[i].1 = Modification::Decreased;
                //self.cells[i.wrapping_sub(1)] -= 1;
                self.add_to(i.wrapping_sub(1), (1 - x) + (1 - y));
                self.add_to(i + 1, x + y);

                avalanche_size += 1;
            }
        }

        avalanche_size
    }

    fn add_to(&mut self, i: usize, add: u8) {
        if i < self.width {
            self.cells[i].0 += add;
            self.cells[i].1 = Modification::Increased;
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui) {
        let texture: &mut egui::TextureHandle = self.texture.get_or_insert_with(|| {
            // Load the texture only once.
            ui.ctx()
                .load_texture("my-image", egui::ColorImage::example(), Default::default())
        });
        let dx = (self.width as f32 / self.zoom) as usize;
        let start = (self.center - dx as f32 / 2.).max(0.);
        let mut text: Vec<Color32> = vec![Color32::BLACK; dx * self.height];

        // (0, w) -> (c - dx, c + dx)
        //  2dx = w / zoom, dx = w / (2 zoom)

        for (i, cell) in self.cells.iter().skip(start as usize).take(dx).enumerate() {
            for h in 0..cell.0 {
                text[dx * (self.height - 1 - h as usize) + i] = cell.1.get_color();
            }
        }

        texture.set(
            ColorImage {
                pixels: text,
                size: [dx, self.height],
                source_size: Vec2 {
                    x: dx as f32,
                    y: self.height as f32,
                },
            },
            TextureOptions::NEAREST,
        );

        // Show the image:
        let resp = ui.add(
            Image::new((texture.id(), Vec2::new(dx as f32, self.height as f32)))
                .maintain_aspect_ratio(true)
                .shrink_to_fit()
                .sense(Sense::drag()),
        );

        self.center -= resp.drag_delta().x / 1000. * self.width as f32 / self.zoom;
        self.zoom += resp.drag_delta().y / 100.;
        self.zoom = self.zoom.clamp(1., 100.);
        self.center = self.center.clamp(
            self.width as f32 / self.zoom / 2.,
            self.width as f32 - self.width as f32 / self.zoom / 2.,
        );
    }
}

/*
/// Call "set_transform" to generate shapes to paint
    pub fn paint(&self, painter: &Painter, transform: RectTransform) {
        for (&i, &color_c) in self.cells.active.iter() {
            let (x, y) = self.cells.index2coord(i);
            let x = x as f32 * 0.9 + (self.cells.width as f32) / 20.0;
            let y = y as f32 * 0.9 + (self.cells.height as f32) / 20.0;
            let point = transform
                * Pos2::new(
                    (x as f32) / (self.cells.width as f32),
                    (y as f32) / (self.cells.height as f32),
                );
            painter.rect_filled(
                Rect::from_center_size(
                    point,
                    transform.scale()
                        * Vec2::new(
                            1.0 / self.cells.width as f32,
                            1.0 / self.cells.height as f32,
                        )
                        * 1.1,
                ),
                Rounding::none(),
                Self::color_gradient(
                    color_c / 4.0,
                    Color32::from_rgb(40, 0, 130),
                    Color32::from_rgb(200, 250, 50),
                ),
            );
        }
    }
*/

#[derive(Debug)]
struct RngWrapper(XorShiftRng);

impl Default for RngWrapper {
    fn default() -> Self {
        Self(XorShiftRng::seed_from_u64(0))
    }
}
