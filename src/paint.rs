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

use super::canvas::Canvas;
use super::element::{Action, GuiElement, SubrectElement};
use super::event::{Event, Keycode, COMMAND, SHIFT};
use super::state::{EditorState, Tool};
use super::tilegrid::TileGrid;
use sdl2::rect::{Point, Rect};
use std::cmp::{max, min};

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
enum ViewSize {
    Small,
    Wide,
    Tall,
    Full,
    Margin,
}

pub struct GridCanvas {
    element: SubrectElement<InnerCanvas>,
}

impl GridCanvas {
    pub fn new(left: i32, top: i32) -> GridCanvas {
        GridCanvas {
            element: SubrectElement::new(
                InnerCanvas::new(),
                Rect::new(left, top, 36 * 16, 25 * 16),
            ),
        }
    }
}

impl GuiElement<EditorState> for GridCanvas {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        self.element.draw(state, canvas);
        let rect = self.element.rect();
        let expanded = Rect::new(
            rect.left() - 2,
            rect.top() - 2,
            rect.width() + 4,
            rect.height() + 4,
        );
        canvas.draw_rect((191, 191, 191, 255), expanded);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        self.element.handle_event(event, state)
    }
}

//===========================================================================//

struct CanvasDrag {
    from_selection: Point,
    from_pixel: Point,
    to_pixel: Point,
}

struct InnerCanvas {
    drag_from_to: Option<CanvasDrag>,
    selection_animation_counter: i32,
    view_size: ViewSize,
}

impl InnerCanvas {
    pub fn new() -> InnerCanvas {
        InnerCanvas {
            drag_from_to: None,
            selection_animation_counter: 0,
            view_size: ViewSize::Full,
        }
    }

    fn mouse_to_row_col(
        &self,
        mouse: Point,
        tilegrid: &TileGrid,
    ) -> Option<(u32, u32)> {
        if mouse.x() < 0 || mouse.y() < 0 {
            return None;
        }
        let scaled = mouse / tilegrid.tile_size() as i32;
        if scaled.x() < 0
            || scaled.x() >= (tilegrid.width() as i32)
            || scaled.y() < 0
            || scaled.y() >= (tilegrid.height() as i32)
        {
            None
        } else {
            Some((scaled.x() as u32, scaled.y() as u32))
        }
    }

    fn clamp_mouse_to_row_col(
        &self,
        mouse: Point,
        tilegrid: &TileGrid,
    ) -> (u32, u32) {
        let scaled = mouse / tilegrid.tile_size() as i32;
        (
            max(0, min(scaled.x(), tilegrid.width() as i32 - 1)) as u32,
            max(0, min(scaled.y(), tilegrid.height() as i32 - 1)) as u32,
        )
    }

    fn dragged_points(
        &self,
        tilegrid: &TileGrid,
    ) -> Option<((u32, u32), (u32, u32))> {
        if let Some(ref drag) = self.drag_from_to {
            let from_point =
                self.clamp_mouse_to_row_col(drag.from_pixel, tilegrid);
            let to_point =
                self.clamp_mouse_to_row_col(drag.to_pixel, tilegrid);
            Some((from_point, to_point))
        } else {
            None
        }
    }

    fn dragged_rect(&self, tilegrid: &TileGrid) -> Option<Rect> {
        if let Some(((from_col, from_row), (to_col, to_row))) =
            self.dragged_points(tilegrid)
        {
            let x = min(from_col, to_col) as i32;
            let y = min(from_row, to_row) as i32;
            let w = ((from_col as i32 - to_col as i32).abs() + 1) as u32;
            let h = ((from_row as i32 - to_row as i32).abs() + 1) as u32;
            Some(Rect::new(x, y, w, h))
        } else {
            None
        }
    }

    fn try_paint(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state.tilegrid())
        {
            let brush = state.brush().clone();
            state.persistent_mutation().tilegrid()[position] = brush;
            true
        } else {
            false
        }
    }

    fn try_eyedrop(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state.tilegrid())
        {
            state.eyedrop(position);
            true
        } else {
            false
        }
    }

    fn try_flood_fill(&self, mouse: Point, state: &mut EditorState) -> bool {
        let start = match self.mouse_to_row_col(mouse, state.tilegrid()) {
            Some(position) => position,
            None => return false,
        };
        let to_tile = state.brush().clone();
        let from_tile = state.tilegrid()[start].clone();
        if from_tile == to_tile {
            return false;
        }
        let mut mutation = state.mutation();
        let tilegrid = mutation.tilegrid();
        tilegrid[start] = to_tile.clone();
        let mut stack: Vec<(u32, u32)> = vec![start];
        while let Some((col, row)) = stack.pop() {
            let mut next: Vec<(u32, u32)> = vec![];
            if col > 0 {
                next.push((col - 1, row));
            }
            if col < tilegrid.width() - 1 {
                next.push((col + 1, row));
            }
            if row > 0 {
                next.push((col, row - 1));
            }
            if row < tilegrid.height() - 1 {
                next.push((col, row + 1));
            }
            for coords in next {
                if tilegrid[coords] == from_tile {
                    tilegrid[coords] = to_tile.clone();
                    stack.push(coords);
                }
            }
        }
        true
    }

    fn try_palette_replace(
        &self,
        mouse: Point,
        state: &mut EditorState,
        swap: bool,
    ) -> bool {
        let start = match self.mouse_to_row_col(mouse, state.tilegrid()) {
            Some(position) => position,
            None => return false,
        };
        let to_tile = state.brush().clone();
        let from_tile = state.tilegrid()[start].clone();
        if from_tile == to_tile {
            return false;
        }
        state.set_brush(from_tile.clone());
        let mut mutation = state.mutation();
        let tilegrid = mutation.tilegrid();
        for y in 0..tilegrid.height() {
            for x in 0..tilegrid.width() {
                let tile = tilegrid[(x, y)].clone();
                if tile == from_tile {
                    tilegrid[(x, y)] = to_tile.clone();
                } else if swap && tile == to_tile {
                    tilegrid[(x, y)] = from_tile.clone();
                }
            }
        }
        true
    }
}

impl GuiElement<EditorState> for InnerCanvas {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let tilegrid = state.tilegrid();
        let horz_margin = 3;
        let vert_margin = 2;
        let row_range = match self.view_size {
            ViewSize::Small | ViewSize::Wide => {
                vert_margin..(tilegrid.height() - vert_margin)
            }
            ViewSize::Tall | ViewSize::Full | ViewSize::Margin => {
                0..tilegrid.height()
            }
        };
        let col_range = match self.view_size {
            ViewSize::Small | ViewSize::Tall => {
                horz_margin..(tilegrid.width() - horz_margin)
            }
            ViewSize::Wide | ViewSize::Full | ViewSize::Margin => {
                0..tilegrid.width()
            }
        };
        canvas.fill_rect(
            tilegrid.background_color(),
            Rect::new(
                (col_range.start * tilegrid.tile_size()) as i32,
                (row_range.start * tilegrid.tile_size()) as i32,
                (col_range.end - col_range.start) * tilegrid.tile_size(),
                (row_range.end - row_range.start) * tilegrid.tile_size(),
            ),
        );
        for row in row_range {
            for col in col_range.clone() {
                if let Some(ref tile) = tilegrid[(col, row)] {
                    canvas.draw_sprite(
                        tile.sprite(),
                        Point::new(
                            (col * tilegrid.tile_size()) as i32,
                            (row * tilegrid.tile_size()) as i32,
                        ),
                    );
                }
            }
        }
        if self.view_size == ViewSize::Margin {
            let rect = Rect::new(
                (horz_margin * tilegrid.tile_size()) as i32,
                (vert_margin * tilegrid.tile_size()) as i32,
                (tilegrid.width() - 2 * horz_margin) * tilegrid.tile_size(),
                (tilegrid.height() - 2 * vert_margin) * tilegrid.tile_size(),
            );
            canvas.draw_rect((63, 63, 63, 255), rect);
        }
        if let Some((ref selected, topleft)) = state.selection() {
            for row in 0..selected.height() {
                for col in 0..selected.width() {
                    if let Some(ref tile) = selected[(col, row)] {
                        let coords = Point::new(col as i32, row as i32);
                        let pos =
                            (coords + topleft) * (tilegrid.tile_size() as i32);
                        canvas.draw_sprite(tile.sprite(), pos);
                    }
                }
            }
            let marquee_rect = Rect::new(
                topleft.x() * (tilegrid.tile_size() as i32),
                topleft.y() * (tilegrid.tile_size() as i32),
                selected.width() * tilegrid.tile_size(),
                selected.height() * tilegrid.tile_size(),
            );
            draw_marquee(
                canvas,
                marquee_rect,
                self.selection_animation_counter,
            );
        } else if let Some(rect) = self.dragged_rect(tilegrid) {
            let marquee_rect = Rect::new(
                rect.x() * (tilegrid.tile_size() as i32),
                rect.y() * (tilegrid.tile_size() as i32),
                rect.width() * tilegrid.tile_size(),
                rect.height() * tilegrid.tile_size(),
            );
            draw_marquee(canvas, marquee_rect, 0);
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        match event {
            &Event::ClockTick => {
                if state.selection().is_some() {
                    self.selection_animation_counter =
                        (self.selection_animation_counter + 1)
                            .rem_euclid(MARQUEE_ANIMATION_MODULUS);
                    Action::redraw().and_continue()
                } else {
                    Action::ignore().and_continue()
                }
            }
            &Event::KeyDown(Keycode::Backspace, _) => {
                if state.selection().is_some() {
                    state.mutation().delete_selection();
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_continue()
                }
            }
            &Event::KeyDown(Keycode::Escape, _) => {
                if state.selection().is_some() {
                    state.mutation().unselect();
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_continue()
                }
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND | SHIFT => {
                self.view_size = match self.view_size {
                    ViewSize::Small => ViewSize::Wide,
                    ViewSize::Wide => ViewSize::Tall,
                    ViewSize::Tall => ViewSize::Full,
                    ViewSize::Full => ViewSize::Margin,
                    ViewSize::Margin => ViewSize::Small,
                };
                Action::redraw().and_stop()
            }
            &Event::MouseDown(pt) => match state.tool() {
                Tool::Eyedropper => {
                    let changed = self.try_eyedrop(pt, state);
                    Action::redraw_if(changed).and_stop()
                }
                Tool::PaintBucket => {
                    let changed = self.try_flood_fill(pt, state);
                    Action::redraw_if(changed).and_stop()
                }
                Tool::PaletteReplace => {
                    let changed = self.try_palette_replace(pt, state, false);
                    Action::redraw_if(changed).and_stop()
                }
                Tool::PaletteSwap => {
                    let changed = self.try_palette_replace(pt, state, true);
                    Action::redraw_if(changed).and_stop()
                }
                Tool::Pencil => {
                    state.reset_persistent_mutation();
                    let changed = self.try_paint(pt, state);
                    Action::redraw_if(changed).and_stop()
                }
                Tool::Select => {
                    let rect = if let Some((ref selected, topleft)) =
                        state.selection()
                    {
                        Some(Rect::new(
                            topleft.x(),
                            topleft.y(),
                            selected.width(),
                            selected.height(),
                        ))
                    } else {
                        None
                    };
                    if let Some(rect) = rect {
                        let tilegrid = state.tilegrid();
                        if !Rect::new(
                            rect.x() * tilegrid.tile_size() as i32,
                            rect.y() * tilegrid.tile_size() as i32,
                            rect.width() * tilegrid.tile_size(),
                            rect.height() * tilegrid.tile_size(),
                        )
                        .contains_point(pt)
                        {
                            state.mutation().unselect();
                        } else {
                            state.reset_persistent_mutation();
                        }
                    }
                    self.drag_from_to = Some(CanvasDrag {
                        from_selection: if let Some(r) = rect {
                            r.top_left()
                        } else {
                            Point::new(0, 0)
                        },
                        from_pixel: pt,
                        to_pixel: pt,
                    });
                    Action::redraw().and_stop()
                }
            },
            &Event::MouseUp => {
                match state.tool() {
                    Tool::Select => {
                        if state.selection().is_none() {
                            if let Some(rect) =
                                self.dragged_rect(state.tilegrid())
                            {
                                state.mutation().select(rect);
                                self.drag_from_to = None;
                                self.selection_animation_counter = 0;
                                return Action::redraw().and_continue();
                            }
                        }
                    }
                    _ => {}
                }
                self.drag_from_to = None;
                Action::ignore().and_continue()
            }
            &Event::MouseDrag(pt) => match state.tool() {
                Tool::Pencil => {
                    let changed = self.try_paint(pt, state);
                    Action::redraw_if(changed).and_continue()
                }
                Tool::Select => {
                    if let Some(ref mut drag) = self.drag_from_to {
                        drag.to_pixel = pt;
                        if state.selection().is_some() {
                            let position = drag.from_selection
                                + (pt - drag.from_pixel)
                                    / state.tilegrid().tile_size() as i32;
                            state
                                .persistent_mutation()
                                .reposition_selection(position);
                        }
                        Action::redraw().and_continue()
                    } else {
                        Action::ignore().and_continue()
                    }
                }
                _ => Action::ignore().and_continue(),
            },
            _ => Action::ignore().and_continue(),
        }
    }
}

//===========================================================================//

const MARQUEE_ANIMATION_MODULUS: i32 = 8;

fn draw_marquee(canvas: &mut Canvas, rect: Rect, anim: i32) {
    canvas.draw_rect((255, 255, 255, 255), rect);
    let color = (0, 0, 0, 255);
    for x in 0..(rect.width() as i32) {
        if (x - anim).rem_euclid(MARQUEE_ANIMATION_MODULUS) < 4 {
            canvas.draw_pixel(color, Point::new(rect.left() + x, rect.top()));
        }
        if (x + anim).rem_euclid(MARQUEE_ANIMATION_MODULUS) < 4 {
            canvas.draw_pixel(
                color,
                Point::new(rect.left() + x, rect.bottom() - 1),
            );
        }
    }
    for y in 0..(rect.height() as i32) {
        if (y + anim).rem_euclid(MARQUEE_ANIMATION_MODULUS) >= 4 {
            canvas.draw_pixel(color, Point::new(rect.left(), rect.top() + y));
        }
        if (y - anim).rem_euclid(MARQUEE_ANIMATION_MODULUS) >= 4 {
            canvas.draw_pixel(
                color,
                Point::new(rect.right() - 1, rect.top() + y),
            );
        }
    }
}

//===========================================================================//
