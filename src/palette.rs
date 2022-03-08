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

use super::canvas::{Canvas, Sprite};
use super::element::{Action, AggregateElement, GuiElement, SubrectElement};
use super::event::{Event, Keycode, NONE};
use super::state::{EditorState, Tool};
use super::tilegrid::{Tile, Tileset};
use sdl2::rect::{Point, Rect};
use std::cmp::max;
use std::rc::Rc;

//===========================================================================//

struct PaletteState {
    tileset: Rc<Tileset>,
    index: usize,
    brush: Option<Tile>,
}

//===========================================================================//

pub struct TilePalette {
    element: SubrectElement<AggregateElement<PaletteState, ()>>,
    tileset_index: usize,
}

impl TilePalette {
    pub fn new(left: i32, top: i32, mut icons: Vec<Sprite>) -> TilePalette {
        icons.truncate(2);
        assert_eq!(icons.len(), 2);
        let right_arrow = icons.pop().unwrap();
        let left_arrow = icons.pop().unwrap();
        let elements: Vec<Box<dyn GuiElement<PaletteState, ()>>> = vec![
            Box::new(SubrectElement::new(
                EraserPicker::new(),
                Rect::new(2, 2, 42, 20),
            )),
            Box::new(SubrectElement::new(
                ArrowButton::new(-1, Keycode::Left, left_arrow),
                Rect::new(4, 26, 16, 16),
            )),
            Box::new(SubrectElement::new(
                ArrowButton::new(1, Keycode::Right, right_arrow),
                Rect::new(26, 26, 16, 16),
            )),
            Box::new(SubrectElement::new(
                InnerPalette::new(),
                Rect::new(0, 42, 46, 258),
            )),
        ];
        TilePalette {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, 46, 300),
            ),
            tileset_index: 0,
        }
    }
}

impl GuiElement<EditorState, ()> for TilePalette {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        let palette_state = PaletteState {
            tileset: state.tilegrid().tileset(),
            index: self.tileset_index,
            brush: state.brush().clone(),
        };
        self.element.draw(&palette_state, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        let mut palette_state = PaletteState {
            tileset: state.tilegrid().tileset(),
            index: self.tileset_index,
            brush: state.brush().clone(),
        };
        let action = self.element.on_event(event, &mut palette_state);
        self.tileset_index = palette_state.index;
        if palette_state.brush != *state.brush() {
            state.set_brush(palette_state.brush);
            if state.tool() == Tool::Select {
                state.set_tool(Tool::Pencil);
            }
        }
        action
    }
}

//===========================================================================//

const SELECTED_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 255);

struct InnerPalette {}

impl InnerPalette {
    fn new() -> InnerPalette {
        InnerPalette {}
    }
}

impl GuiElement<PaletteState, ()> for InnerPalette {
    fn draw(&self, state: &PaletteState, canvas: &mut Canvas) {
        for (index, tile) in state.tileset.tiles(state.index).enumerate() {
            let left = 4 + 22 * (index % 2) as i32;
            let top = 4 + 22 * (index / 2) as i32;
            canvas.draw_sprite(tile.sprite(), Point::new(left, top));
            if Some(tile) == state.brush {
                canvas.draw_rect(
                    SELECTED_COLOR,
                    Rect::new(left - 2, top - 2, 20, 20),
                );
            }
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut PaletteState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(pt) => {
                let mut found = None;
                for (index, tile) in
                    state.tileset.tiles(state.index).enumerate()
                {
                    let left = 4 + 22 * (index % 2) as i32;
                    let top = 4 + 22 * (index / 2) as i32;
                    let rect = Rect::new(left, top, 16, 16);
                    if rect.contains_point(pt) {
                        found = Some(Some(tile));
                        break;
                    }
                }
                if let Some(brush) = found {
                    state.brush = brush;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            _ => Action::ignore()
        }
    }
}

//===========================================================================//

struct EraserPicker {}

impl EraserPicker {
    fn new() -> EraserPicker {
        EraserPicker {}
    }
}

impl GuiElement<PaletteState, ()> for EraserPicker {
    fn draw(&self, state: &PaletteState, canvas: &mut Canvas) {
        let rect = canvas.rect();
        canvas.draw_rect((0, 0, 0, 255), shrink(rect, 2));
        canvas.draw_rect((0, 0, 0, 255), shrink(rect, 4));
        canvas.draw_rect((0, 0, 0, 255), shrink(rect, 6));
        if state.brush.is_none() {
            canvas.draw_rect(SELECTED_COLOR, rect);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut PaletteState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(_) => {
                state.brush = None;
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct ArrowButton {
    icon: Sprite,
    key: Keycode,
    delta: i32,
}

impl ArrowButton {
    fn new(delta: i32, key: Keycode, icon: Sprite) -> ArrowButton {
        ArrowButton { icon, key, delta }
    }

    fn increment(&self, state: &mut PaletteState) -> Action<()> {
        let num_filenames = state.tileset.num_filenames();
        if num_filenames > 0 {
            state.index = (state.index as i32 + self.delta)
                .rem_euclid(num_filenames as i32)
                as usize;
            Action::redraw().and_stop()
        } else {
            Action::ignore()
        }
    }
}

impl GuiElement<PaletteState, ()> for ArrowButton {
    fn draw(&self, _: &PaletteState, canvas: &mut Canvas) {
        canvas.draw_sprite(&self.icon, Point::new(0, 0));
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut PaletteState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(_) => self.increment(state),
            &Event::KeyDown(key, kmod) if key == self.key && kmod == NONE => {
                self.increment(state)
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

fn shrink(rect: Rect, by: i32) -> Rect {
    Rect::new(
        rect.x() + by,
        rect.y() + by,
        max((rect.width() as i32) - 2 * by, 0) as u32,
        max((rect.height() as i32) - 2 * by, 0) as u32,
    )
}

//===========================================================================//
