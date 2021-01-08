// +--------------------------------------------------------------------------+
// | Copyright 2016 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Linoleum.                                           |
// |                                                                          |
// | Linoleum is free software: you can redistribute it and/or modify it      |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Linoleum is distributed in the hope that it will be useful, but WITHOUT  |
// | ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or    |
// | FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License    |
// | for details.                                                             |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Linoleum.  If not, see <http://www.gnu.org/licenses/>.              |
// +--------------------------------------------------------------------------+

use super::canvas::{Canvas, Font};
use super::element::{Action, GuiElement, SubrectElement};
use super::event::{Event, Keycode};
use super::state::{EditorState, Mode};
use sdl2::rect::{Point, Rect};
use std::cmp;
use std::rc::Rc;

//===========================================================================//

const LABEL_WIDTH: i32 = 40;

pub struct TextBox {
    font: Rc<Font>,
}

impl TextBox {
    pub fn new(font: Rc<Font>) -> TextBox {
        TextBox { font }
    }
}

impl GuiElement<String> for TextBox {
    fn draw(&self, text: &String, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let text_width = self.font.text_width(text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        render_string(canvas, &self.font, text_left, 4, text);
        canvas.draw_rect((255, 255, 255, 255), rect);
    }

    fn handle_event(&mut self, event: &Event, text: &mut String) -> Action {
        match event {
            &Event::KeyDown(Keycode::Backspace, _) => {
                Action::redraw_if(text.pop().is_some()).and_stop()
            }
            &Event::KeyDown(_, _) => Action::ignore().and_stop(),
            &Event::TextInput(ref input) => {
                text.push_str(input);
                Action::redraw().and_stop()
            }
            _ => Action::ignore().and_continue(),
        }
    }
}

//===========================================================================//

pub struct ModalTextBox {
    left: i32,
    top: i32,
    font: Rc<Font>,
    element: SubrectElement<TextBox>,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32, font: Rc<Font>) -> ModalTextBox {
        ModalTextBox {
            left,
            top,
            font: font.clone(),
            element: SubrectElement::new(
                TextBox::new(font),
                Rect::new(
                    left + LABEL_WIDTH,
                    top,
                    (676 - LABEL_WIDTH) as u32,
                    18,
                ),
            ),
        }
    }
}

impl GuiElement<EditorState> for ModalTextBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let label = match *state.mode() {
            Mode::Edit => {
                self.element.draw(state.filepath(), canvas);
                "Path:"
            }
            Mode::LoadFile(ref text) => {
                self.element.draw(text, canvas);
                "Load:"
            }
            Mode::SaveAs(ref text) => {
                self.element.draw(text, canvas);
                "Save:"
            }
            Mode::ChangeColor(ref text) => {
                self.element.draw(text, canvas);
                "Color:"
            }
            Mode::ChangeTiles(ref text) => {
                self.element.draw(text, canvas);
                "Tiles:"
            }
        };
        let text_width = self.font.text_width(label);
        render_string(
            canvas,
            &self.font,
            self.left + LABEL_WIDTH - text_width - 2,
            self.top + 4,
            label,
        );
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                if state.mode_cancel() {
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_continue()
                }
            }
            &Event::KeyDown(Keycode::Return, _) => {
                Action::ignore().and_stop_if(state.enqueue_mode_perform())
            }
            _ => match *state.mode_mut() {
                Mode::Edit => Action::ignore().and_continue(),
                Mode::LoadFile(ref mut text)
                | Mode::SaveAs(ref mut text)
                | Mode::ChangeColor(ref mut text)
                | Mode::ChangeTiles(ref mut text) => {
                    self.element.handle_event(event, text)
                }
            },
        }
    }
}

//===========================================================================//

fn render_string(
    canvas: &mut Canvas,
    font: &Font,
    left: i32,
    top: i32,
    string: &str,
) {
    canvas.draw_text(font, Point::new(left, top + font.baseline()), string);
}

//===========================================================================//
