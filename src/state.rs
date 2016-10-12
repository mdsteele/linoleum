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

use sdl2::rect::{Point, Rect};
use std::fs::File;
use std::io;
use std::mem;
use std::rc::Rc;
use super::canvas::Window;
use super::tilegrid::{GRID_NUM_COLS, GRID_NUM_ROWS, SubGrid, Tile, TileGrid,
                      Tileset};

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Tool {
    Eyedropper,
    PaintBucket,
    Pencil,
    Select,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Mode {
    Edit,
    LoadFile(String),
    SaveAs(String),
    ChangeColor(String),
}

// ========================================================================= //

const MAX_UNDOS: usize = 100;

#[derive(Clone)]
struct Snapshot {
    tilegrid: Rc<TileGrid>,
    selection: Option<(Rc<SubGrid>, Point)>,
    unsaved: bool,
}

// ========================================================================= //

pub struct EditorState {
    mode: Mode,
    filepath: String,
    current: Snapshot,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    clipboard: Option<(Rc<SubGrid>, Point)>,
    tool: Tool,
    prev_tool: Tool,
    brush: Option<Tile>,
    persistent_mutation_active: bool,
    should_mode_perform: bool,
}

impl EditorState {
    pub fn new(filepath: String, tileset: Tileset) -> EditorState {
        EditorState {
            mode: Mode::Edit,
            filepath: filepath,
            current: Snapshot {
                tilegrid: Rc::new(TileGrid::new(tileset)),
                selection: None,
                unsaved: true,
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            brush: None,
            persistent_mutation_active: false,
            should_mode_perform: false,
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn mode_mut(&mut self) -> &mut Mode {
        &mut self.mode
    }

    pub fn filepath(&self) -> &String {
        &self.filepath
    }

    pub fn tilegrid(&self) -> &TileGrid {
        &self.current.tilegrid
    }

    pub fn is_unsaved(&self) -> bool {
        self.current.unsaved
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.unselect_if_necessary();
            self.prev_tool = self.tool;
            self.tool = tool;
        }
    }

    pub fn brush(&self) -> &Option<Tile> {
        &self.brush
    }

    pub fn set_brush(&mut self, tile: Option<Tile>) {
        self.brush = tile;
    }

    pub fn eyedrop(&mut self, position: (u32, u32)) {
        self.brush = self.current.tilegrid[position].clone();
        if self.tool == Tool::Eyedropper {
            self.tool = if self.prev_tool == Tool::Select {
                Tool::Pencil
            } else {
                self.prev_tool
            };
        }
    }

    pub fn selection(&self) -> Option<(&SubGrid, Point)> {
        match self.current.selection {
            Some((ref subgrid, position)) => Some((&subgrid, position)),
            None => None,
        }
    }

    fn unselect_if_necessary(&mut self) {
        self.reset_persistent_mutation();
        if self.selection().is_some() {
            self.mutation().unselect();
        }
    }

    pub fn mutation(&mut self) -> Mutation {
        self.push_change();
        self.current.unsaved = true;
        Mutation { state: self }
    }

    pub fn persistent_mutation(&mut self) -> Mutation {
        if !self.persistent_mutation_active {
            self.push_change();
            self.persistent_mutation_active = true;
        }
        self.current.unsaved = true;
        Mutation { state: self }
    }

    pub fn reset_persistent_mutation(&mut self) {
        self.persistent_mutation_active = false;
    }

    fn push_change(&mut self) {
        self.reset_persistent_mutation();
        self.redo_stack.clear();
        self.undo_stack.push(self.current.clone());
        if self.undo_stack.len() > MAX_UNDOS {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(mut snapshot) = self.undo_stack.pop() {
            mem::swap(&mut snapshot, &mut self.current);
            self.redo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut snapshot) = self.redo_stack.pop() {
            mem::swap(&mut snapshot, &mut self.current);
            self.undo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&mut self) -> io::Result<()> {
        self.unselect_if_necessary();
        let mut file = try!(File::create(&self.filepath));
        try!(self.tilegrid().save(&mut file));
        self.current.unsaved = false;
        for snapshot in self.undo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        for snapshot in self.redo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        Ok(())
    }

    pub fn begin_load_file(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::LoadFile(self.filepath.clone());
            true
        } else {
            false
        }
    }

    pub fn begin_save_as(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::SaveAs(self.filepath.clone());
            true
        } else {
            false
        }
    }

    pub fn begin_change_color(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            let (r, g, b, _) = self.tilegrid().background_color();
            self.mode = Mode::ChangeColor(format!("{},{},{}", r, g, b));
            true
        } else {
            false
        }
    }

    pub fn mode_cancel(&mut self) -> bool {
        match self.mode {
            Mode::Edit => false,
            _ => {
                self.mode = Mode::Edit;
                true
            }
        }
    }

    pub fn enqueue_mode_perform(&mut self) -> bool {
        match self.mode {
            Mode::Edit => false,
            _ => {
                self.should_mode_perform = true;
                true
            }
        }
    }

    pub fn mode_perform_if_necessary(&mut self, window: &Window) -> bool {
        if !self.should_mode_perform {
            return false;
        }
        self.should_mode_perform = false;
        match self.mode.clone() {
            Mode::Edit => false,
            Mode::LoadFile(path) => {
                match TileGrid::load_from_path(window,
                                               self.tilegrid()
                                                   .tileset()
                                                   .dirpath(),
                                               &path) {
                    Ok(tilegrid) => {
                        self.filepath = path;
                        self.current.tilegrid = Rc::new(tilegrid);
                        self.current.unsaved = false;
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.brush = None;
                        self.persistent_mutation_active = false;
                        self.mode = Mode::Edit;
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::SaveAs(mut path) => {
                mem::swap(&mut path, &mut self.filepath);
                match self.save_to_file() {
                    Ok(()) => {
                        self.mode = Mode::Edit;
                        true
                    }
                    Err(_) => {
                        mem::swap(&mut path, &mut self.filepath);
                        false
                    }
                }
            }
            Mode::ChangeColor(text) => {
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
                self.mutation().set_background_color(red, green, blue);
                self.mode = Mode::Edit;
                true
            }
        }
    }
}

// ========================================================================= //

pub struct Mutation<'a> {
    state: &'a mut EditorState,
}

impl<'a> Mutation<'a> {
    pub fn tilegrid(&mut self) -> &mut TileGrid {
        Rc::make_mut(&mut self.state.current.tilegrid)
    }

    pub fn set_background_color(&mut self, red: u8, green: u8, blue: u8) {
        self.tilegrid().set_background_color(red, green, blue);
    }

    pub fn select(&mut self, rect: Rect) {
        self.unselect();
        let subgrid = self.tilegrid().cut_subgrid(rect);
        self.state.current.selection = Some((Rc::new(subgrid),
                                             rect.top_left()));
        self.state.prev_tool = self.state.tool;
        self.state.tool = Tool::Select;
    }

    pub fn select_all(&mut self) {
        self.select(Rect::new(0, 0, GRID_NUM_COLS, GRID_NUM_ROWS));
    }

    pub fn unselect(&mut self) {
        if let Some((grid, position)) = self.state.current.selection.take() {
            self.tilegrid().paste_subgrid(&grid, position);
        }
    }

    pub fn delete_selection(&mut self) {
        self.state.current.selection = None;
    }

    pub fn cut_selection(&mut self) {
        if self.state.current.selection.is_some() {
            self.state.clipboard = self.state.current.selection.take();
        } else {
            let rect = Rect::new(0, 0, GRID_NUM_COLS, GRID_NUM_ROWS);
            let subgrid = self.tilegrid().cut_subgrid(rect);
            self.state.clipboard = Some((Rc::new(subgrid), Point::new(0, 0)));
        }
    }

    pub fn copy_selection(&mut self) {
        if self.state.current.selection.is_some() {
            self.state.clipboard = self.state.current.selection.clone();
        } else {
            let rect = Rect::new(0, 0, GRID_NUM_COLS, GRID_NUM_ROWS);
            let subgrid = self.state.tilegrid().copy_subgrid(rect);
            self.state.clipboard = Some((Rc::new(subgrid), Point::new(0, 0)));
        }
    }

    pub fn paste_selection(&mut self) {
        if self.state.clipboard.is_some() {
            self.unselect();
            self.state.current.selection = self.state.clipboard.clone();
            self.state.tool = Tool::Select;
        }
    }

    pub fn reposition_selection(&mut self, new_position: Point) {
        if let Some((_, ref mut position)) = self.state.current.selection {
            *position = new_position;
        }
    }
}

// ========================================================================= //
