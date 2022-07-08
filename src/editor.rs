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

use crate::canvas::Canvas;
use crate::canvas::{Font, Sprite, Window};
use crate::coords::{CoordsIndicator, CoordsKind};
use crate::element::{Action, AggregateElement, GuiElement};
use crate::event::{Event, Keycode, COMMAND, SHIFT};
use crate::paint::GridCanvas;
use crate::palette::TilePalette;
use crate::state::EditorState;
use crate::textbox::{ModalTextBox, Mode};
use crate::tilegrid::TileGrid;
use crate::toolbox::Toolbox;
use crate::unsaved::UnsavedIndicator;
use std::rc::Rc;

//===========================================================================//

// These limits are currently arbitrary:
const MAX_GRID_WIDTH: u32 = 100;
const MAX_GRID_HEIGHT: u32 = 100;

//===========================================================================//

pub struct EditorView {
    aggregate: AggregateElement<EditorState, ()>,
    textbox: ModalTextBox,
}

impl EditorView {
    pub fn new(
        tool_icons: Vec<Sprite>,
        arrow_icons: Vec<Sprite>,
        unsaved_icon: Sprite,
        font: Rc<Font>,
    ) -> EditorView {
        let elements: Vec<Box<dyn GuiElement<EditorState, ()>>> = vec![
            Box::new(Toolbox::new(10, 34, tool_icons)),
            Box::new(TilePalette::new(10, 116, arrow_icons)),
            Box::new(GridCanvas::new(72, 34, font.clone())),
            Box::new(UnsavedIndicator::new(10, 10, unsaved_icon)),
            Box::new(CoordsIndicator::new(
                658,
                34,
                font.clone(),
                CoordsKind::TileDec,
            )),
            Box::new(CoordsIndicator::new(
                658,
                334,
                font.clone(),
                CoordsKind::PixelDec,
            )),
            Box::new(CoordsIndicator::new(
                658,
                392,
                font.clone(),
                CoordsKind::PixelHex,
            )),
        ];
        EditorView {
            aggregate: AggregateElement::new(elements),
            textbox: ModalTextBox::new(32, 8, font.clone()),
        }
    }

    fn begin_load_file(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.textbox
                .set_mode(Mode::LoadFile, state.filepath().to_string());
            true
        } else {
            false
        }
    }

    fn begin_save_as(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.textbox.set_mode(Mode::SaveAs, state.filepath().to_string());
            true
        } else {
            false
        }
    }

    fn begin_resize_grid(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.textbox.set_mode(
                Mode::Resize,
                format!(
                    "{}x{}",
                    state.tilegrid().width(),
                    state.tilegrid().height()
                ),
            );
            true
        } else {
            false
        }
    }

    fn begin_change_color(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            let (r, g, b, _) = state.tilegrid().background_color();
            self.textbox
                .set_mode(Mode::ChangeColor, format!("{},{},{}", r, g, b));
            true
        } else {
            false
        }
    }

    fn begin_change_tiles(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            let mut string = String::new();
            for filename in state.tilegrid().tileset().filenames() {
                if !string.is_empty() {
                    string.push(',');
                }
                string.push_str(&filename);
            }
            self.textbox.set_mode(Mode::ChangeTiles, string);
            true
        } else {
            false
        }
    }

    pub fn mode_perform(
        &mut self,
        window: &Window,
        state: &mut EditorState,
        mode: Mode,
        text: String,
    ) -> bool {
        let success = self.mode_perform_internal(window, state, mode, text);
        if success {
            self.textbox.clear_mode();
        }
        success
    }

    fn mode_perform_internal(
        &mut self,
        window: &Window,
        state: &mut EditorState,
        mode: Mode,
        text: String,
    ) -> bool {
        match mode {
            Mode::Edit => false,
            Mode::LoadFile => {
                match TileGrid::load_from_path(
                    window,
                    state.tilegrid().tileset().dirpath(),
                    &text,
                ) {
                    Ok(tilegrid) => {
                        state.load_tilegrid(text, tilegrid);
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::SaveAs => {
                let old = state.swap_filepath(text);
                match state.save_to_file() {
                    Ok(()) => true,
                    Err(_) => {
                        state.swap_filepath(old);
                        false
                    }
                }
            }
            Mode::Resize => {
                let pieces: Vec<&str> = text.split('x').collect();
                if pieces.len() != 2 {
                    return false;
                }
                let new_width = match pieces[0].parse::<u32>() {
                    Ok(width) => width,
                    Err(_) => return false,
                };
                let new_height = match pieces[1].parse::<u32>() {
                    Ok(height) => height,
                    Err(_) => return false,
                };
                if new_width == 0
                    || new_height == 0
                    || new_width > MAX_GRID_WIDTH
                    || new_height > MAX_GRID_HEIGHT
                {
                    return false;
                }
                state.mutation().resize_grid(new_width, new_height);
                true
            }
            Mode::ChangeColor => {
                let pieces: Vec<&str> = text.split(',').collect();
                if pieces.len() != 3 {
                    return false;
                }
                let red = match pieces[0].parse::<u8>() {
                    Ok(red) => red,
                    Err(_) => return false,
                };
                let green = match pieces[1].parse::<u8>() {
                    Ok(green) => green,
                    Err(_) => return false,
                };
                let blue = match pieces[2].parse::<u8>() {
                    Ok(blue) => blue,
                    Err(_) => return false,
                };
                state.mutation().set_background_color(red, green, blue);
                true
            }
            Mode::ChangeTiles => {
                let pieces: Vec<&str> = text.split(',').collect();
                if pieces.len() < 1 {
                    return false;
                }
                state.mutation().set_tile_filenames(window, pieces).is_ok()
            }
        }
    }
}

impl GuiElement<EditorState, (Mode, String)> for EditorView {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let rect = canvas.rect();
        canvas.draw_rect((127, 127, 127, 127), rect);
        self.aggregate.draw(state, canvas);
        self.textbox.draw(state, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<(Mode, String)> {
        match event {
            &Event::KeyDown(Keycode::A, kmod) if kmod == COMMAND => {
                state.mutation().select_all();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::B, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_change_color(state)).and_stop()
            }
            &Event::KeyDown(Keycode::C, kmod) if kmod == COMMAND => {
                state.mutation().copy_selection();
                Action::ignore().and_stop()
            }
            &Event::KeyDown(Keycode::H, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().flip_selection_horz();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::O, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_load_file(state)).and_stop()
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_resize_grid(state)).and_stop()
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND => {
                state.save_to_file().unwrap();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_save_as(state)).and_stop()
            }
            &Event::KeyDown(Keycode::T, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_change_tiles(state)).and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND => {
                state.mutation().paste_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().flip_selection_vert();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::X, kmod) if kmod == COMMAND => {
                state.mutation().cut_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.undo()).and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.redo()).and_stop()
            }
            _ => {
                let mut action = self.textbox.on_event(event, state);
                if !action.should_stop() {
                    let subaaction = self.aggregate.on_event(event, state);
                    action.merge(subaaction.but_no_value());
                }
                action
            }
        }
    }
}

//===========================================================================//
