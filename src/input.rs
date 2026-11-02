//! Ввод мыши в логических координатах.

use macroquad::prelude::*;

pub struct Input {
    pub mouse: Vec2,
    pub pressed: bool,
    pub released: bool,
    pub down: bool,
    pub press_pos: Vec2,
    pub wheel: f32,
    /// Ввод перехвачен модальным окном.
    pub blocked: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            mouse: Vec2::ZERO,
            pressed: false,
            released: false,
            down: false,
            press_pos: Vec2::ZERO,
            wheel: 0.0,
            blocked: false,
        }
    }

    pub fn update(&mut self, mouse: Vec2) {
        self.mouse = mouse;
        self.pressed = is_mouse_button_pressed(MouseButton::Left);
        self.released = is_mouse_button_released(MouseButton::Left);
        self.down = is_mouse_button_down(MouseButton::Left);
        if self.pressed {
            self.press_pos = mouse;
        }
        let (_, wy) = mouse_wheel();
        self.wheel = wy.clamp(-3.0, 3.0);
        self.blocked = false;
    }

    pub fn hover(&self, r: Rect) -> bool {
        !self.blocked && r.contains(self.mouse)
    }

    /// Клик: кнопку нажали и отпустили внутри прямоугольника.
    pub fn clicked(&self, r: Rect) -> bool {
        !self.blocked && self.released && r.contains(self.mouse) && r.contains(self.press_pos)
    }

    pub fn pressed_in(&self, r: Rect) -> bool {
        !self.blocked && self.pressed && r.contains(self.mouse)
    }
}
