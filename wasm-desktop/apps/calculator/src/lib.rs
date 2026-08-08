use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet};

pub struct CalculatorApp {
    display: String,
    accumulator: f64,
    pending_op: Option<char>,
    fresh_input: bool,
}

impl CalculatorApp {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            accumulator: 0.0,
            pending_op: None,
            fresh_input: true,
        }
    }

    fn apply(op: char, a: f64, b: f64) -> f64 {
        match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => {
                if b != 0.0 {
                    a / b
                } else {
                    f64::NAN
                }
            }
            _ => b,
        }
    }

    fn format_num(n: f64) -> String {
        if n.is_nan() {
            "Error".to_string()
        } else if n.fract() == 0.0 && n.abs() < 1e15 {
            format!("{n:.0}")
        } else {
            format!("{n}")
        }
    }

    fn press(&mut self, label: &str) {
        match label {
            "" => {}
            "C" => *self = Self::new(),
            "." => {
                if self.fresh_input {
                    self.display = "0.".to_string();
                    self.fresh_input = false;
                } else if !self.display.contains('.') {
                    self.display.push('.');
                }
            }
            "=" => {
                if let Some(op) = self.pending_op.take() {
                    let rhs: f64 = self.display.parse().unwrap_or(0.0);
                    self.accumulator = Self::apply(op, self.accumulator, rhs);
                    self.display = Self::format_num(self.accumulator);
                }
                self.fresh_input = true;
            }
            "÷" | "×" | "−" | "+" => {
                let op = match label {
                    "÷" => '/',
                    "×" => '*',
                    "−" => '-',
                    _ => '+',
                };
                let rhs: f64 = self.display.parse().unwrap_or(0.0);
                self.accumulator = match self.pending_op {
                    Some(pending) => Self::apply(pending, self.accumulator, rhs),
                    None => rhs,
                };
                self.display = Self::format_num(self.accumulator);
                self.pending_op = Some(op);
                self.fresh_input = true;
            }
            digit => {
                if self.fresh_input || self.display == "0" {
                    self.display = digit.to_string();
                    self.fresh_input = false;
                } else {
                    self.display.push_str(digit);
                }
            }
        }
    }
}

impl DesktopApplet for CalculatorApp {
    fn id(&self) -> &'static str {
        "calculator"
    }

    fn label(&self) -> &'static str {
        "Calculator"
    }

    fn window_title(&self) -> &'static str {
        "Calculator"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(96, 96, 102)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();

        let body = Rect::from_center_size(badge.center(), Vec2::new(w * 0.56, h * 0.72));
        painter.rect_filled(body, CornerRadius::from(3.0), Color32::WHITE);

        let screen = Rect::from_min_size(
            egui::pos2(body.left() + w * 0.06, body.top() + h * 0.06),
            Vec2::new(body.width() - w * 0.12, h * 0.16),
        );
        painter.rect_filled(screen, CornerRadius::from(1.0), self.accent_color());

        for row in 0..3 {
            for col in 0..3 {
                let x = body.left() + w * 0.09 + col as f32 * w * 0.16;
                let y = body.top() + h * 0.32 + row as f32 * h * 0.13;
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(x, y), Vec2::new(w * 0.1, h * 0.08)),
                    CornerRadius::from(1.0),
                    self.accent_color(),
                );
            }
        }
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(240.0, 320.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        egui::Frame::new()
            .fill(Color32::from_rgb(28, 28, 30))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width().max(200.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(&self.display)
                            .size(30.0)
                            .color(Color32::WHITE)
                            .monospace(),
                    );
                });
                ui.add_space(10.0);

                let rows: [[&str; 4]; 5] = [
                    ["C", "", "", "÷"],
                    ["7", "8", "9", "×"],
                    ["4", "5", "6", "−"],
                    ["1", "2", "3", "+"],
                    ["0", "", ".", "="],
                ];

                let mut pressed: Option<&'static str> = None;

                egui::Grid::new("calc-grid")
                    .num_columns(4)
                    .spacing(Vec2::new(8.0, 8.0))
                    .show(ui, |ui| {
                        for row in rows {
                            for label in row {
                                if label.is_empty() {
                                    ui.label("");
                                    continue;
                                }
                                let is_op = matches!(label, "÷" | "×" | "−" | "+" | "=" | "C");
                                let button = egui::Button::new(
                                    RichText::new(label).size(16.0).color(Color32::WHITE),
                                )
                                .fill(if is_op {
                                    self.accent_color()
                                } else {
                                    Color32::from_rgb(64, 64, 68)
                                })
                                .min_size(Vec2::new(42.0, 36.0));

                                if ui.add(button).clicked() {
                                    pressed = Some(label);
                                }
                            }
                            ui.end_row();
                        }
                    });

                if let Some(label) = pressed {
                    self.press(label);
                }
            });
    }
}
