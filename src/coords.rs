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

use sdl2::rect::Point;
use std::rc::Rc;
use super::canvas::{Canvas, Font};
use super::element::{Action, GuiElement};
use super::event::Event;
use super::tilegrid::TILE_SIZE;
use super::state::EditorState;

// ========================================================================= //

pub struct CoordsIndicator {
    topleft: Point,
    font: Rc<Font>,
}

impl CoordsIndicator {
    pub fn new(left: i32, top: i32, font: Rc<Font>) -> CoordsIndicator {
        CoordsIndicator {
            topleft: Point::new(left, top),
            font: font,
        }
    }
}

impl GuiElement<EditorState> for CoordsIndicator {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        if let Some((subgrid, position)) = state.selection() {
            let size = TILE_SIZE as i32;
            let left = position.x() * size;
            let top = position.y() * size;
            let right = left + subgrid.width() as i32 * size;
            let bottom = top + subgrid.height() as i32 * size;
            canvas.draw_text(&self.font,
                             self.topleft + Point::new(15, 10),
                             &format!("{}", top));
            canvas.draw_text(&self.font,
                             self.topleft + Point::new(0, 25),
                             &format!("{}", left));
            canvas.draw_text(&self.font,
                             self.topleft + Point::new(30, 25),
                             &format!("{}", right));
            canvas.draw_text(&self.font,
                             self.topleft + Point::new(15, 40),
                             &format!("{}", bottom));
        }
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> Action {
        Action::ignore().and_continue()
    }
}

// ========================================================================= //
