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
use super::state::EditorState;
use sdl2::rect::{Point, Rect};
use std::cmp;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::rc::Rc;

//===========================================================================//

const CURSOR_ON_FRAMES: u32 = 3;
const CURSOR_OFF_FRAMES: u32 = 3;

const LABEL_WIDTH: i32 = 40;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Edit,
    LoadFile,
    SaveAs,
    Resize,
    ChangeColor,
    ChangeTiles,
}

impl Mode {
    fn is_file_picker(self) -> bool {
        match self {
            Mode::LoadFile | Mode::SaveAs => true,
            _ => false,
        }
    }
}

//===========================================================================//

struct TextBox {
    font: Rc<Font>,
    byte_index: usize,
    cursor_blink: u32,
    text: String,
}

impl TextBox {
    pub fn new(font: Rc<Font>) -> TextBox {
        TextBox { font, byte_index: 0, cursor_blink: 0, text: String::new() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.byte_index = text.len();
        self.text = text;
        self.cursor_blink = 0;
    }
}

impl GuiElement<(), ()> for TextBox {
    fn draw(&self, _: &(), canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let text_width = self.font.text_width(&self.text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        render_string(canvas, &self.font, text_left, 4, &self.text);
        canvas.draw_rect((255, 255, 255, 255), rect);
        if self.cursor_blink < CURSOR_ON_FRAMES {
            let cursor_x = text_left
                + self.font.text_width(&self.text[..self.byte_index]);
            let cursor_rect =
                Rect::new(cursor_x, rect.y() + 3, 1, rect.height() - 6);
            canvas.fill_rect((255, 255, 0, 255), cursor_rect);
        }
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<()> {
        match event {
            &Event::ClockTick => {
                let was_on = self.cursor_blink < CURSOR_ON_FRAMES;
                self.cursor_blink = (self.cursor_blink + 1)
                    % (CURSOR_ON_FRAMES + CURSOR_OFF_FRAMES);
                let is_on = self.cursor_blink < CURSOR_ON_FRAMES;
                Action::redraw_if(was_on != is_on)
            }
            &Event::KeyDown(Keycode::Backspace, _) => {
                if self.byte_index > 0 {
                    let rest = self.text.split_off(self.byte_index);
                    self.text.pop();
                    self.byte_index = self.text.len();
                    self.text.push_str(&rest);
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Up, _) => {
                if self.byte_index > 0 {
                    self.byte_index = 0;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Down, _) => {
                if self.byte_index < self.text.len() {
                    self.byte_index = self.text.len();
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Left, _) => {
                if self.byte_index > 0 {
                    let mut new_byte_index = self.byte_index - 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index -= 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Right, _) => {
                if self.byte_index < self.text.len() {
                    let mut new_byte_index = self.byte_index + 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index += 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(_, _) => Action::ignore().and_stop(),
            &Event::TextInput(ref input) => {
                self.text.insert_str(self.byte_index, input);
                self.byte_index += input.len();
                self.cursor_blink = 0;
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct MatchesPanel {
    left: i32,
    top: i32,
    font: Rc<Font>,
    matches: Vec<String>,
}

impl MatchesPanel {
    fn new(left: i32, top: i32, font: Rc<Font>) -> MatchesPanel {
        MatchesPanel { left, top, font, matches: Vec::new() }
    }

    fn set_matches(&mut self, matches: Vec<String>) {
        self.matches = matches;
    }

    fn clear_matches(&mut self) {
        self.matches.clear();
    }
}

impl GuiElement<(), ()> for MatchesPanel {
    fn draw(&self, _: &(), canvas: &mut Canvas) {
        if !self.matches.is_empty() {
            let rect = Rect::new(
                self.left,
                self.top,
                360,
                4 + 14 * (self.matches.len() as u32),
            );
            canvas.fill_rect((128, 128, 128, 255), rect);
            canvas.draw_rect((255, 255, 255, 255), rect);
            for (row, string) in self.matches.iter().enumerate() {
                render_string(
                    canvas,
                    &self.font,
                    self.left + 4,
                    self.top + 4 + 14 * (row as i32),
                    string,
                );
            }
        }
    }

    fn on_event(&mut self, _: &Event, _: &mut ()) -> Action<()> {
        Action::ignore()
    }
}

//===========================================================================//

pub struct ModalTextBox {
    left: i32,
    top: i32,
    font: Rc<Font>,
    mode: Mode,
    textbox: SubrectElement<TextBox>,
    matches_panel: MatchesPanel,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32, font: Rc<Font>) -> ModalTextBox {
        ModalTextBox {
            left,
            top,
            font: font.clone(),
            mode: Mode::Edit,
            textbox: SubrectElement::new(
                TextBox::new(font.clone()),
                Rect::new(
                    left + LABEL_WIDTH,
                    top,
                    (676 - LABEL_WIDTH) as u32,
                    18,
                ),
            ),
            matches_panel: MatchesPanel::new(
                left + LABEL_WIDTH,
                top + 20,
                font.clone(),
            ),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode, text: String) {
        self.mode = mode;
        self.textbox.inner_mut().set_text(text);
        self.matches_panel.clear_matches();
    }

    pub fn clear_mode(&mut self) {
        self.mode = Mode::Edit;
        self.textbox.inner_mut().set_text(String::new());
        self.matches_panel.clear_matches();
    }

    fn tab_complete(&mut self) -> bool {
        match tab_complete_path(self.textbox.inner().text()) {
            Ok((path, matches)) => {
                self.textbox.inner_mut().set_text(path);
                if matches.len() > 1 {
                    self.matches_panel.set_matches(matches);
                } else {
                    self.matches_panel.clear_matches();
                }
                true
            }
            Err(_) => false,
        }
    }
}

impl GuiElement<EditorState, (Mode, String)> for ModalTextBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        if self.mode == Mode::Edit {
            render_string(
                canvas,
                &self.font,
                self.left + LABEL_WIDTH + 4,
                self.top + 4,
                state.filepath(),
            );
        } else {
            self.textbox.draw(&(), canvas);
            if self.mode.is_file_picker() {
                self.matches_panel.draw(&(), canvas);
            }
        }
        let label = match self.mode {
            Mode::Edit => "Path:",
            Mode::LoadFile => "Load:",
            Mode::SaveAs => "Save:",
            Mode::Resize => "Size:",
            Mode::ChangeColor => "Color:",
            Mode::ChangeTiles => "Tiles:",
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

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut EditorState,
    ) -> Action<(Mode, String)> {
        if self.mode == Mode::Edit {
            return Action::ignore();
        }
        let mut action = match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                self.clear_mode();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Return, _) => {
                let text = self.textbox.inner().text().to_string();
                Action::redraw().and_return((self.mode, text))
            }
            &Event::KeyDown(Keycode::Tab, _) => {
                let redraw = self.mode.is_file_picker() && self.tab_complete();
                Action::redraw_if(redraw).and_stop()
            }
            _ => Action::ignore(),
        };
        if !action.should_stop() {
            let subaction = self.textbox.on_event(event, &mut ());
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            action = action.and_stop();
        }
        action
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

fn tab_complete_path(path_string: &str) -> io::Result<(String, Vec<String>)> {
    let path = Path::new(path_string);
    let (dir, prefix): (&Path, &str) = if path_string.ends_with('/') {
        (path, "")
    } else {
        (
            path.parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, ""))?,
            path.file_name().map(OsStr::to_str).unwrap_or(None).unwrap_or(""),
        )
    };

    let mut file_names = Vec::<String>::new();
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let file_name = entry.file_name().to_str().unwrap_or("").to_string();
        if file_name.starts_with(prefix) {
            file_names.push(file_name);
        }
    }

    if let Some(first) = file_names.first() {
        let mut completed = String::new();
        for chr in first.chars() {
            let mut candidate = completed.clone();
            candidate.push(chr);
            if !file_names.iter().all(|name| name.starts_with(&candidate)) {
                break;
            }
            completed = candidate;
        }
        let mut completed_path = dir.join(completed);
        if completed_path.is_dir() {
            completed_path.push("");
        }
        Ok((
            completed_path
                .into_os_string()
                .into_string()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, ""))?,
            file_names,
        ))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, ""))
    }
}

//===========================================================================//
