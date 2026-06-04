use std::time::Instant;

use egui::Vec2;
use egui_plot::{Plot, PlotPoints, Points};

use crate::physics::{Modification, Simulation1D};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MannaApp {
    simulation: Simulation1D,
    #[serde(skip)]
    paused: bool,
    #[serde(skip)]
    avalanche_sizes: Vec<u32>,
    #[serde(skip)]
    points: Vec<(f64, f64)>,

    #[serde(skip)]
    time: f64,

    collapse_short_time: bool,
    collapse_large_time: bool,
    delay: f64,
    steps_per_frame: usize,
    // skip_first: usize,
    state: SimState,

    cur_avalanche_size: u32,

    #[serde(skip)]
    last_updated: Option<Instant>,

    zoom_above_one: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
enum SimState {
    #[default]
    Growing,
    Bouncing,
}

impl MannaApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app
    }
}

impl eframe::App for MannaApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    /*fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

    }*/

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::Panel::top("panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        // TODO: FIX!
                        // _frame.close();
                        ui.close();
                    }
                });
            });
        });

        //egui::Window::new("Parameters").show(ctx, |ui| {
        egui::Panel::right("Parameters").show_inside(ui, |ui| {
            ui.checkbox(&mut self.paused, "Paused");
            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.paused = !self.paused;
            }

            ui.end_row();

            ui.add(egui::Slider::new(&mut self.delay, 0.0..=1.0).text("Delay between events"));

            ui.end_row();
            // ui.add(egui::Slider::new(&mut self.skip_first, 0..=1000).text("Skip first"));
            ui.checkbox(
                &mut self.collapse_short_time,
                "Collapse small times dynamics\n (may lead to lag on large sim)",
            );
            ui.checkbox(
                &mut self.collapse_large_time,
                "Collapse large times dynamics",
            );
            ui.add(
                egui::Slider::new(&mut self.steps_per_frame, 1..=100)
                    .text("Small dynamics steps per iteration"),
            );
            let old_width = self.simulation.width;
            ui.add(egui::Slider::new(&mut self.simulation.width, 3..=5_000).text("Width first"));
            if self.simulation.width != old_width {
                self.simulation.cells = vec![(0, Modification::Ignored); self.simulation.width];
                self.avalanche_sizes = vec![];
                self.points = vec![];
            }

            ui.checkbox(
                &mut self.simulation.fixed_point,
                "Fixed point (center) generation",
            );

            if ui.button("Clear data").clicked() {
                self.points = vec![];
                self.avalanche_sizes = vec![];
            }

            // ui.label(format!("{:?}", self.avalanche_sizes));
            /*ui.checkbox(&mut self.collisions, "Collisions");
            ui.end_row();
            ui.add(egui::Slider::new(&mut self.measure_time, 0.01..=1.0).text("Measuring time"));
            ui.add(egui::Slider::new(&mut self.temperature, 0.0..=3.0).text("Temperature"));
            ui.add(egui::Slider::new(&mut self.balls_n, 0..=1000).text("Balls number"));
            ui.add(egui::Slider::new(&mut self.radius, 0.0..=0.03).text("Ball radius"));
            ui.add(egui::Slider::new(&mut self.filter_height, 0.0..=1.0).text("Filter height"));
            ui.add(egui::Slider::new(&mut self.wall_width, 0.0..=0.1).text("Wall width"));
            */

            /*
            egui::ComboBox::from_label("Filter type:")
                .selected_text(match self.filter_type {
                    MaxwellType::Diode => "Diode",
                    MaxwellType::Temperature {..} => "Temperature",
                    MaxwellType::Tennis => "Tennis",
                    MaxwellType::Empty => "Empty",
                    MaxwellType::PhaseConserving {..} => "Phase conserving",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_type, MaxwellType::Diode, "Diode");
                    ui.selectable_value(&mut self.filter_type, MaxwellType::Temperature { t: self.filter_temperature}, "Temperature");
                    ui.selectable_value(&mut self.filter_type, MaxwellType::Tennis, "Tennis");
                    ui.selectable_value(&mut self.filter_type, MaxwellType::PhaseConserving { c: self.filter_constant }, "Phase conserving");
                    ui.selectable_value(&mut self.filter_type, MaxwellType::Empty, "Empty");
                }
            );

            if let MaxwellType::Temperature { t } = &mut self.filter_type{
                ui.add(egui::Slider::new(t, 0.0..=5.0).text("Filter temperature"));
            }
            else if let MaxwellType::PhaseConserving { c } = &mut self.filter_type{
                ui.add(egui::Slider::new(c, 0.0..=1.0).text("Filter constant"));
            }


            if ui.button("Regenerate").clicked() {
                self.simulation.random_initiation(self.balls_n, self.temperature, self.radius, self.filter_height, self.filter_type, self.collisions, self.wall_width);
                self.points.clear();
                self.time = 0.0;
            }

            let (left_count, right_symbol) = self.simulation.structure.count_balls(&self.simulation);
            ui.label(format!("\nLeft side: {} balls,\nRight side: {} balls", left_count, right_symbol));
            density = (left_count as f64)/((left_count + right_symbol) as f64)*100.0;
            ui.label(format!("Left chamber density: {:.1} %", density));
            ui.add_space(10.0);
            */

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("© ");
                    ui.hyperlink_to("sitandr", "https://github.com/sitandr");
                    ui.label(", 2026");
                });
            });
        });

        // want to plot something like f(x) = x⁻¹
        // 1 -
        if true {
            if self.avalanche_sizes.len() % 10 == 0 && self.avalanche_sizes.len() > 0 {
                let max = self.avalanche_sizes.iter().max().unwrap();

                let n_steps = (self.avalanche_sizes.len() as f64).sqrt().ceil();

                // a (n + 1)² / 2 - a n² / 2 = a (n + 1); a (n + 1)² / 2 = max; a = 2 max / (n + 1)²

                let a = 2. * *max as f64 / (n_steps).powi(2);

                let mut points = vec![0.; n_steps as usize + 1];

                for s in &self.avalanche_sizes {
                    let bin_n = (2. * (*s as f64) / a).sqrt().floor();

                    points[bin_n as usize] += 1. / (a * (bin_n + 1.));
                }

                self.points = points
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| **x > 0.)
                    .map(|(i, x)| ((a * (i as f64 + 0.5).powi(2) / 2.).log10(), x.log10()))
                    .collect();
            }
            //egui::Window::new("Left density/time").show(ctx, |ui| {
            egui::Panel::bottom("Graphs")
                .resizable(true)
                .show_inside(ui, |ui| {
                    Plot::new("data")
                        .x_axis_label("Size, 10^n")
                        .x_axis_formatter(|mark, _| format!("10^{:}", mark.value))
                        .y_axis_formatter(|mark, _| format!("10^{:.0}", mark.value))
                        .y_axis_label("Number / size")
                        .min_size(Vec2::new(200., 150.))
                        .show(ui, |plot_ui| {
                            plot_ui.points(
                                Points::new(
                                    "Avalanche size",
                                    self.points
                                        .iter()
                                        .map(|&(x, p)| [x, p])
                                        .collect::<PlotPoints<'_>>(),
                                )
                                .radius(3.),
                            )
                        });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if !self.paused
                && (self.delay == 0.
                    || self
                        .last_updated
                        .is_none_or(|i| i.elapsed().as_secs_f64() >= self.delay))
            {
                match self.state {
                    SimState::Growing => loop {
                        if self.simulation.random_step() {
                            if self.cur_avalanche_size > 0 {
                                self.avalanche_sizes.push(self.cur_avalanche_size);
                            }
                            self.cur_avalanche_size = 0;
                            self.state = SimState::Bouncing;
                            break;
                        } else if !self.collapse_large_time {
                            break;
                        }
                    },
                    SimState::Bouncing => {
                        let mut counter = 0;
                        loop {
                            let size = self.simulation.step();
                            self.cur_avalanche_size += size;
                            if self.steps_per_frame > 1 && !self.collapse_short_time {
                                counter += 1;
                                if counter > self.steps_per_frame {
                                    break;
                                }
                            }
                            if size == 0 {
                                self.state = SimState::Growing;
                                break;
                            }
                            if !self.collapse_short_time && self.steps_per_frame == 1 {
                                break;
                            }
                        }
                    }
                }
                ui.ctx().request_repaint_after_secs(self.delay as f32);
                if self.delay != 0. && cfg!(not(target_arch = "wasm32")) {
                    self.last_updated = Some(Instant::now());
                }
            } else if !self.paused {
                ui.ctx().request_repaint_after_secs(
                    self.delay as f32 - self.last_updated.unwrap().elapsed().as_secs_f32(),
                );
            }

            // egui::Area::new(egui::Id::new("display area")).default_size([500., 200.]).show(ui, |ui| {});
            self.simulation.display(ui);

            // painter.rect_stroke(rect, 1.0, Stroke::new(1.0, Color32::from_gray(16)), StrokeKind::Middle);
            // Make sure we allocate what we used (everything)
            // ui.expand_to_include_rect(painter.clip_rect());
            egui::warn_if_debug_build(ui);
        });
    }
}
