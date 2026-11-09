use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{circle::Circle, prelude::*, vec2::Vec2};

#[derive(Copy, Clone)]
struct DragState {
    index: usize,
    offset: Vec2,
}

pub struct Game {
    pub circles: Vec<Circle>,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub padding: f64,
    pub friction: f32,
    pub restitution: f32,
    pub gravity: f32,
    mouse: Option<Vec2>,
    selected: Option<usize>,
    drag: Option<DragState>,
    drag_velocity: Vec2,
    last_drag_target: Option<Vec2>,
    fps: f32,
    fps_frames: u32,
    fps_elapsed: f32,
}

pub struct GameWindow {
    start: Vec2,
    end: Vec2,
}

fn resolve_collision(
    velocity_i: Vec2,
    velocity_j: Vec2,
    inv_mass_i: f32,
    inv_mass_j: f32,
    normal: Vec2,
) -> (Vec2, Vec2) {
    let relative_velocity = velocity_j - velocity_i;
    let velocity_along_normal = relative_velocity.x * normal.x + relative_velocity.y * normal.y;

    if velocity_along_normal >= 0.0 {
        return (velocity_i, velocity_j);
    }

    // Perfectly elastic impulse, scaled by each circle's inverse mass.
    let impulse = -2.0 * velocity_along_normal / (inv_mass_i + inv_mass_j);
    (
        velocity_i - normal * (impulse * inv_mass_i),
        velocity_j + normal * (impulse * inv_mass_j),
    )
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
            mouse: None,
            selected: None,
            drag: None,
            drag_velocity: Vec2 { x: 0.0, y: 0.0 },
            last_drag_target: None,
            fps: 0.0,
            fps_frames: 0,
            fps_elapsed: 0.0,
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

        if let Some(circle) = self.selected.and_then(|index| self.circles.get(index)) {
            circle.render_outline(&self.ctx);
        }

        self.ctx.set_fill_style_str("#333333");
        self.ctx.set_font("20px sans-serif");
        self.ctx
            .fill_text(&format!("Balls: {}", self.circles.len()), 16.0, 32.0)
            .unwrap();
        self.ctx
            .fill_text(&format!("FPS: {:.0}", self.fps), 16.0, 56.0)
            .unwrap();
    }

    pub fn update_fps(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        self.fps_frames += 1;
        self.fps_elapsed += dt;

        if self.fps_elapsed >= 0.5 {
            self.fps = self.fps_frames as f32 / self.fps_elapsed;
            self.fps_frames = 0;
            self.fps_elapsed = 0.0;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

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

        let drag_index = self.drag.map(|drag| drag.index);
        let drag_target = self.drag_target();
        let drag_velocity = self.drag_velocity;

        if let (Some(target), Some(index)) = (drag_target, drag_index)
            && let Some(circle) = self.circles.get_mut(index)
        {
            circle.pos = target;
            circle.prev_pos = target - drag_velocity * dt;
        }

        let acceleration = Vec2 {
            x: 0.0,
            y: 100.0 * self.gravity,
        };

        for (index, circle) in self.circles.iter_mut().enumerate() {
            if drag_index == Some(index) {
                continue;
            }
            circle.integrate(dt, acceleration);
        }

        for i in 0..self.circles.len() {
            for j in 0..i {
                let (pos_i, prev_i, pos_j, prev_j, radius_i, radius_j) = {
                    let circle_i = &self.circles[i];
                    let circle_j = &self.circles[j];
                    (
                        circle_i.pos,
                        circle_i.prev_pos,
                        circle_j.pos,
                        circle_j.prev_pos,
                        circle_i.radius,
                        circle_j.radius,
                    )
                };
                let itoj = pos_j - pos_i;
                let dist_sq = itoj.magsq();
                let radii_sum = radius_i + radius_j;
                if dist_sq >= (radii_sum * radii_sum) {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let itoj_norm = if dist > 0.0 {
                    itoj / dist
                } else {
                    let relative_velocity = (pos_j - prev_j) - (pos_i - prev_i);
                    let relative_speed = relative_velocity.magsq().sqrt();
                    if relative_speed > 0.0 {
                        -relative_velocity / relative_speed
                    } else {
                        Vec2 { x: 1.0, y: 0.0 }
                    }
                };

                // Mass scales with area, so the impulse and positional
                // correction respect each circle's size. A dragged circle is
                // treated as infinitely heavy so it pushes others around.
                let inv_mass_i = if drag_index == Some(i) {
                    0.0
                } else {
                    1.0 / (radius_i * radius_i)
                };
                let inv_mass_j = if drag_index == Some(j) {
                    0.0
                } else {
                    1.0 / (radius_j * radius_j)
                };
                let inv_mass_sum = inv_mass_i + inv_mass_j;
                let overlap = radii_sum - dist;

                let new_pos_i = pos_i - itoj_norm * (overlap * inv_mass_i / inv_mass_sum);
                let new_pos_j = pos_j + itoj_norm * (overlap * inv_mass_j / inv_mass_sum);

                // Verlet velocities are implicit in the position history.
                let velocity_i = (pos_i - prev_i) / dt;
                let velocity_j = (pos_j - prev_j) / dt;
                let (velocity_i, velocity_j) =
                    resolve_collision(velocity_i, velocity_j, inv_mass_i, inv_mass_j, itoj_norm);

                self.circles[i].pos = new_pos_i;
                self.circles[i].prev_pos = new_pos_i - velocity_i * dt;
                self.circles[j].pos = new_pos_j;
                self.circles[j].prev_pos = new_pos_j - velocity_j * dt;
            }
        }

        for (index, circle) in self.circles.iter_mut().enumerate() {
            if drag_index == Some(index) {
                continue;
            }

            let mut velocity = circle.pos - circle.prev_pos;

            if (circle.pos.x - circle.radius) < game_window.start.x {
                if velocity.x < 0.0 {
                    velocity.x *= -self.restitution;
                }
                velocity.y *= 1.0 - self.friction;
                circle.pos.x = game_window.start.x + circle.radius;
            }

            if (circle.pos.y - circle.radius) < game_window.start.y {
                if velocity.y < 0.0 {
                    velocity.y *= -self.restitution;
                }
                velocity.x *= 1.0 - self.friction;
                circle.pos.y = game_window.start.y + circle.radius;
            }

            if (circle.pos.x + circle.radius) > game_window.end.x {
                if velocity.x > 0.0 {
                    velocity.x *= -self.restitution;
                }
                velocity.y *= 1.0 - self.friction;
                circle.pos.x = game_window.end.x - circle.radius;
            }

            if (circle.pos.y + circle.radius) > game_window.end.y {
                if velocity.y >= 0.0 {
                    velocity.y *= -self.restitution;
                }
                velocity.x *= 1.0 - self.friction;
                circle.pos.y = game_window.end.y - circle.radius;
            }

            circle.prev_pos = circle.pos - velocity;
        }
    }

    pub fn spawn(&mut self, circle: Circle) {
        self.circles.push(circle);
    }

    fn circle_at(&self, point: Vec2) -> Option<usize> {
        self.circles
            .iter()
            .enumerate()
            .rev()
            .find(|(_, circle)| circle.contains(point))
            .map(|(index, _)| index)
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse = Some(Vec2 { x, y });
        self.update_hover();
    }

    pub fn update_hover(&mut self) {
        if self.drag.is_none() {
            self.selected = self.mouse.and_then(|point| self.circle_at(point));
        }
    }

    fn drag_target(&self) -> Option<Vec2> {
        let drag = self.drag?;
        let mouse = self.mouse?;
        Some(mouse + drag.offset)
    }

    pub fn update_drag_velocity(&mut self, dt: f32) {
        let Some(target) = self.drag_target() else {
            self.drag_velocity = Vec2 { x: 0.0, y: 0.0 };
            self.last_drag_target = None;
            return;
        };

        if let Some(previous) = self.last_drag_target
            && dt > 0.0
        {
            self.drag_velocity = (target - previous) / dt;
        }

        self.last_drag_target = Some(target);
    }

    pub fn start_drag(&mut self) {
        let Some(point) = self.mouse else {
            return;
        };

        let index = match self.circle_at(point) {
            Some(index) => index,
            None => {
                self.spawn(Circle::new(point, 40.0));
                self.circles.len() - 1
            }
        };

        self.selected = Some(index);
        self.drag = Some(DragState {
            index,
            offset: self.circles[index].pos - point,
        });
        self.drag_velocity = Vec2 { x: 0.0, y: 0.0 };
        self.last_drag_target = Some(self.circles[index].pos);
    }

    pub fn end_drag(&mut self) {
        self.drag = None;
        self.drag_velocity = Vec2 { x: 0.0, y: 0.0 };
        self.last_drag_target = None;
        self.selected = self.mouse.and_then(|point| self.circle_at(point));
    }

    pub fn delete_selected(&mut self) {
        let Some(index) = self.selected else {
            return;
        };

        self.circles.remove(index);
        self.drag = None;
        self.drag_velocity = Vec2 { x: 0.0, y: 0.0 };
        self.last_drag_target = None;
        self.selected = self.mouse.and_then(|point| self.circle_at(point));
    }

    pub fn resize_selected(&mut self, delta: f32) {
        let Some(index) = self.selected else {
            return;
        };

        let circle = &mut self.circles[index];
        circle.radius = (circle.radius + delta).clamp(10.0, 300.0);
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_collision;
    use crate::vec2::Vec2;

    #[test]
    fn head_on_collision_exchanges_normal_velocity() {
        let (velocity_i, velocity_j) = resolve_collision(
            Vec2 { x: 1.0, y: 0.0 },
            Vec2 { x: -1.0, y: 0.0 },
            1.0,
            1.0,
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x + 1.0).abs() < f32::EPSILON);
        assert!((velocity_j.x - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn separating_circles_do_not_receive_another_impulse() {
        let (velocity_i, velocity_j) = resolve_collision(
            Vec2 { x: -1.0, y: 0.0 },
            Vec2 { x: 1.0, y: 0.0 },
            1.0,
            1.0,
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x + 1.0).abs() < f32::EPSILON);
        assert!((velocity_j.x - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lighter_circle_bounces_off_a_heavier_one() {
        let (velocity_i, velocity_j) = resolve_collision(
            Vec2 { x: 1.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
            1.0,
            1.0 / 4.0,
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x + 0.6).abs() < 1.0e-5);
        assert!((velocity_j.x - 0.4).abs() < 1.0e-5);
    }

    #[test]
    fn infinitely_heavy_circle_pushes_a_stationary_one() {
        let (velocity_i, velocity_j) = resolve_collision(
            Vec2 { x: 1.0, y: 0.0 },
            Vec2 { x: 0.0, y: 0.0 },
            0.0,
            1.0,
            Vec2 { x: 1.0, y: 0.0 },
        );

        assert!((velocity_i.x - 1.0).abs() < f32::EPSILON);
        assert!((velocity_j.x - 2.0).abs() < 1.0e-5);
    }

    #[test]
    fn equal_mass_collision_conserves_momentum() {
        let (velocity_i, velocity_j) = resolve_collision(
            Vec2 { x: 3.0, y: -1.0 },
            Vec2 { x: -1.0, y: 2.0 },
            1.0 / 2.0,
            1.0 / 2.0,
            Vec2 { x: 1.0, y: 0.0 },
        );

        // Momentum along the collision normal is unchanged.
        assert!((2.0 * velocity_i.x + 2.0 * velocity_j.x - 4.0).abs() < 1.0e-5);
    }
}
