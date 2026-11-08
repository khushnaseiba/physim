use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{circle::Circle, prelude::*, vec2::Vec2};

pub struct Game {
    pub circles: Vec<Circle>,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub padding: f64,
    pub friction: f32,
    pub restitution: f32,
    pub gravity: f32,
}

pub struct GameWindow {
    start: Vec2,
    end: Vec2,
}

fn resolve_equal_mass_collision(velocity_i: Vec2, velocity_j: Vec2, normal: Vec2) -> (Vec2, Vec2) {
    let relative_velocity = velocity_j - velocity_i;
    let velocity_along_normal = relative_velocity.x * normal.x + relative_velocity.y * normal.y;

    if velocity_along_normal >= 0.0 {
        return (velocity_i, velocity_j);
    }

    let impulse = normal * velocity_along_normal;
    (velocity_i + impulse, velocity_j - impulse)
}

impl Game {
    pub fn new(
        canvas: HtmlCanvasElement,
        padding: f64,
        friction: f32,
        restitution: f32,
        gravity: f32,
    ) -> Game {
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        Self {
            circles: Vec::new(),
            canvas,
            ctx,
            padding,
            friction,
            restitution,
            gravity,
        }
    }

    pub fn render(&self) {
        let window = window();

        let game_width = window.inner_width().unwrap().as_f64().unwrap();
        let game_height = window.inner_height().unwrap().as_f64().unwrap();

        let canvas_width = self.canvas.width();
        let canvas_height = self.canvas.height();

        if (canvas_width as f64) != game_width {
            self.canvas.set_width(game_width as u32);
        }

        if (canvas_height as f64) != game_height {
            self.canvas.set_height(game_height as u32);
        }

        self.ctx.clear_rect(0.0, 0.0, game_width, game_height);
        self.ctx.set_fill_style_str("#787777");
        self.ctx.fill_rect(
            self.padding,
            self.padding,
            game_width - 2.0 * self.padding,
            game_height - 2.0 * self.padding,
        );

        for circle in &self.circles {
            circle.render(&self.ctx);
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let window = window();

        let game_width = window.inner_width().unwrap().as_f64().unwrap();
        let game_height = window.inner_height().unwrap().as_f64().unwrap();

        let game_window = GameWindow {
            start: Vec2 {
                x: self.padding as f32,
                y: self.padding as f32,
            },
            end: Vec2 {
                x: (game_width - self.padding) as f32,
                y: (game_height - self.padding) as f32,
            },
        };

        for i in 0..self.circles.len() {
            for j in 0..i {
                let (pos_i, pos_j, velocity_i, velocity_j, radius_i, radius_j) = {
                    let circle_i = &self.circles[i];
                    let circle_j = &self.circles[j];
                    (
                        circle_i.pos,
                        circle_j.pos,
                        circle_i.velocity,
                        circle_j.velocity,
                        circle_i.radius,
                        circle_j.radius,
                    )
                };
                let itoj = pos_j - pos_i;
                let dist_sq = itoj.magsq();
                let radii_sum = radius_i + radius_j;
                if dist_sq < (radii_sum * radii_sum) {
                    let avg = (pos_i + pos_j) / 2.0;
                    let dist = dist_sq.sqrt();
                    let itoj_norm = if dist > 0.0 {
                        itoj / dist
                    } else {
                        let relative_velocity = velocity_j - velocity_i;
                        let relative_speed = relative_velocity.magsq().sqrt();
                        if relative_speed > 0.0 {
                            -relative_velocity / relative_speed
                        } else {
                            Vec2 { x: 1.0, y: 0.0 }
                        }
                    };
                    self.circles[i].pos = avg + itoj_norm * -radius_i;
                    self.circles[j].pos = avg + itoj_norm * radius_j;

                    let (velocity_i, velocity_j) =
                        resolve_equal_mass_collision(velocity_i, velocity_j, itoj_norm);
                    self.circles[i].velocity = velocity_i;
                    self.circles[j].velocity = velocity_j;
                }
            }
        }

        for circle in &mut self.circles {
            circle.velocity += Vec2 {
                x: 0.0,
                y: 100.0 * self.gravity * dt,
            };
            circle.tick(dt);

            if (circle.pos.x - circle.radius) < game_window.start.x {
                if circle.velocity.x < 0.0 {
                    circle.velocity.x = -circle.velocity.x * self.restitution;
                }
                circle.velocity.y = circle.velocity.y * (1.0 - self.friction);
                circle.pos.x = game_window.start.x + circle.radius;
            }

            if (circle.pos.y - circle.radius) < game_window.start.y {
                if circle.velocity.y < 0.0 {
                    circle.velocity.y = -circle.velocity.y * self.restitution;
                }
                circle.velocity.x = circle.velocity.x * (1.0 - self.friction);
                circle.pos.y = game_window.start.y + circle.radius;
            }

            if (circle.pos.x + circle.radius) > game_window.end.x {
                if circle.velocity.x > 0.0 {
                    circle.velocity.x = -circle.velocity.x * self.restitution;
                }
                circle.velocity.y = circle.velocity.y * (1.0 - self.friction);
                circle.pos.x = game_window.end.x - circle.radius;
            }

            if (circle.pos.y + circle.radius) > game_window.end.y {
                if circle.velocity.y >= 0.0 {
                    circle.velocity.y = -circle.velocity.y * self.restitution;
                }
                circle.velocity.x = circle.velocity.x * (1.0 - self.friction);
                circle.pos.y = game_window.end.y - circle.radius;
            }
        }
    }

    pub fn spawn(&mut self, circle: Circle) {
        self.circles.push(circle);
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_equal_mass_collision;
    use crate::vec2::Vec2;

    #[test]
    fn head_on_collision_exchanges_normal_velocity() {
        let (velocity_i, velocity_j) = resolve_equal_mass_collision(
            Vec2 { x: 1.0, y: 0.0 },
            Vec2 { x: -1.0, y: 0.0 },
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x + 1.0).abs() < f32::EPSILON);
        assert!((velocity_j.x - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn separating_circles_do_not_receive_another_impulse() {
        let (velocity_i, velocity_j) = resolve_equal_mass_collision(
            Vec2 { x: -1.0, y: 0.0 },
            Vec2 { x: 1.0, y: 0.0 },
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x + 1.0).abs() < f32::EPSILON);
        assert!((velocity_j.x - 1.0).abs() < f32::EPSILON);
    }
}
