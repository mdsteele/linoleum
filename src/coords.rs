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
use super::element::{Action, GuiElement};
use super::event::Event;
use super::state::EditorState;
use sdl2::rect::Point;
use std::rc::Rc;

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum CoordsKind {
    PixelDec,
    PixelHex,
    TileDec,
}

impl CoordsKind {
    fn format(self, value: i32, tile_size: i32) -> String {
        match self {
            CoordsKind::PixelDec => format!("{}", value * tile_size),
            CoordsKind::PixelHex => format!("{:03x}", value * tile_size),
            CoordsKind::TileDec => format!("{}", value),
        }
    }
}

//===========================================================================//

pub struct CoordsIndicator {
    topleft: Point,
    font: Rc<Font>,
    kind: CoordsKind,
}

impl CoordsIndicator {
    pub fn new(
        left: i32,
        top: i32,
        font: Rc<Font>,
        kind: CoordsKind,
    ) -> CoordsIndicator {
        CoordsIndicator { topleft: Point::new(left, top), font, kind }
    }
}

impl GuiElement<EditorState, ()> for CoordsIndicator {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let tile_size = state.tilegrid().tile_size() as i32;
        if let Some((subgrid, position)) = state.selection() {
            let left = position.x();
            let top = position.y();
            let right = left + subgrid.width() as i32;
            let bottom = top + subgrid.height() as i32;
            canvas.draw_text(
                &self.font,
                self.topleft + Point::new(15, 10),
                &self.kind.format(top, tile_size),
            );
            canvas.draw_text(
                &self.font,
                self.topleft + Point::new(0, 25),
                &self.kind.format(left, tile_size),
            );
            canvas.draw_text(
                &self.font,
                self.topleft + Point::new(30, 25),
                &self.kind.format(right, tile_size),
            );
            canvas.draw_text(
                &self.font,
                self.topleft + Point::new(15, 40),
                &self.kind.format(bottom, tile_size),
            );
        }
    }

    fn on_event(&mut self, _: &Event, _: &mut EditorState) -> Action<()> {
        Action::ignore()
    }
}

//===========================================================================//
