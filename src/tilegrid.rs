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
use std::cmp::{max, min, Ordering};
use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use super::canvas::{Canvas, Sprite};
use super::util;

// ========================================================================= //

pub struct Tileset {
    dirpath: PathBuf,
    tiles: Vec<(String, Vec<Rc<Sprite>>)>,
}

impl Tileset {
    pub fn load(canvas: &Canvas,
                dirpath: PathBuf,
                filenames: &[String])
                -> io::Result<Tileset> {
        let mut tiles = vec![];
        for filename in filenames {
            let path = dirpath.join(filename);
            let images = try!(util::load_ahi_from_file(&path.to_str()
                                                            .unwrap()
                                                            .to_string()));
            let mut sprites = vec![];
            for image in images {
                let sprite = canvas.new_sprite(&image);
                sprites.push(Rc::new(sprite));
            }
            tiles.push((filename.to_string(), sprites));
        }
        Ok(Tileset {
            dirpath: dirpath,
            tiles: tiles,
        })
    }

    pub fn dirpath(&self) -> &Path {
        &self.dirpath
    }

    pub fn num_filenames(&self) -> usize {
        self.tiles.len()
    }

    pub fn filenames(&self) -> Filenames {
        Filenames {
            tileset: self,
            index: 0,
        }
    }

    pub fn tiles(&self, file_index: usize) -> Tiles {
        Tiles {
            tileset: self,
            file_index: file_index,
            tile_index: 0,
        }
    }

    pub fn get(&self, file_index: usize, tile_index: usize) -> Option<Tile> {
        if file_index >= self.tiles.len() {
            return None;
        }
        let (ref filename, ref sprites) = self.tiles[file_index];
        if tile_index >= sprites.len() {
            return None;
        }
        Some(Tile {
            filename: filename.clone(),
            index: tile_index,
            sprite: sprites[tile_index].clone(),
        })
    }
}

pub struct Filenames<'a> {
    tileset: &'a Tileset,
    index: usize,
}

impl<'a> Iterator for Filenames<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.index >= self.tileset.tiles.len() {
            None
        } else {
            let (ref filename, _) = self.tileset.tiles[self.index];
            self.index += 1;
            Some(filename.clone())
        }
    }
}

pub struct Tiles<'a> {
    tileset: &'a Tileset,
    file_index: usize,
    tile_index: usize,
}

impl<'a> Iterator for Tiles<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.file_index >= self.tileset.tiles.len() {
            return None;
        }
        let (ref filename, ref tiles) = self.tileset.tiles[self.file_index];
        if self.tile_index >= tiles.len() {
            return None;
        }
        let tile = Tile {
            filename: filename.clone(),
            index: self.tile_index,
            sprite: tiles[self.tile_index].clone(),
        };
        self.tile_index += 1;
        return Some(tile);
    }
}

// ========================================================================= //

pub const TILE_SIZE: u32 = 16;

#[derive(Clone)]
pub struct Tile {
    filename: String,
    index: usize,
    sprite: Rc<Sprite>,
}

impl Tile {
    pub fn sprite(&self) -> &Sprite {
        self.sprite.as_ref()
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.filename == other.filename && self.index == other.index
    }
}

impl Eq for Tile {}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Tile) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Tile) -> Ordering {
        (&self.filename, self.index).cmp(&(&other.filename, other.index))
    }
}

// ========================================================================= //

#[derive(Clone)]
pub struct SubGrid {
    width: u32,
    height: u32,
    grid: Vec<Option<Tile>>,
}

impl SubGrid {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Index<(u32, u32)> for SubGrid {
    type Output = Option<Tile>;
    fn index(&self, (col, row): (u32, u32)) -> &Option<Tile> {
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &self.grid[(row * self.width + col) as usize]
    }
}

// ========================================================================= //

pub const GRID_NUM_COLS: u32 = 36;
pub const GRID_NUM_ROWS: u32 = 24;

#[derive(Clone)]
pub struct TileGrid {
    background_color: (u8, u8, u8),
    tileset: Rc<Tileset>,
    grid: Vec<Option<Tile>>,
}

impl TileGrid {
    pub fn new(tileset: Tileset) -> TileGrid {
        TileGrid {
            background_color: (15, 15, 15),
            tileset: Rc::new(tileset),
            grid: vec![None; (GRID_NUM_ROWS * GRID_NUM_COLS) as usize],
        }
    }

    pub fn background_color(&self) -> (u8, u8, u8, u8) {
        let (r, g, b) = self.background_color;
        (r, g, b, 255)
    }

    pub fn set_background_color(&mut self, red: u8, green: u8, blue: u8) {
        self.background_color = (red, green, blue);
    }

    pub fn tileset(&self) -> Rc<Tileset> {
        self.tileset.clone()
    }

    pub fn copy_subgrid(&self, rect: Rect) -> SubGrid {
        let mut grid = Vec::new();
        let start_col = max(0, rect.left()) as u32;
        let end_col = min(GRID_NUM_COLS as i32, rect.right()) as u32;
        let start_row = max(0, rect.top()) as u32;
        let end_row = min(GRID_NUM_ROWS as i32, rect.bottom()) as u32;
        for row in start_row..end_row {
            for col in start_col..end_col {
                grid.push(self[(col, row)].clone());
            }
        }
        SubGrid {
            width: end_col - start_col,
            height: end_row - start_row,
            grid: grid,
        }
    }

    pub fn cut_subgrid(&mut self, rect: Rect) -> SubGrid {
        let mut grid = Vec::new();
        let start_col = max(0, rect.left()) as u32;
        let end_col = min(GRID_NUM_COLS as i32, rect.right()) as u32;
        let start_row = max(0, rect.top()) as u32;
        let end_row = min(GRID_NUM_ROWS as i32, rect.bottom()) as u32;
        for row in start_row..end_row {
            for col in start_col..end_col {
                grid.push(self[(col, row)].clone());
                self[(col, row)] = None;
            }
        }
        SubGrid {
            width: end_col - start_col,
            height: end_row - start_row,
            grid: grid,
        }
    }

    pub fn paste_subgrid(&mut self, subgrid: &SubGrid, topleft: Point) {
        let src_start_row = min(max(0, -topleft.y()) as u32, subgrid.height);
        let src_start_col = min(max(0, -topleft.x()) as u32, subgrid.width);
        let dest_start_row = min(max(0, topleft.y()) as u32, GRID_NUM_ROWS);
        let dest_start_col = min(max(0, topleft.x()) as u32, GRID_NUM_COLS);
        let num_rows = min(subgrid.height - src_start_row,
                           GRID_NUM_ROWS - dest_start_row);
        let num_cols = min(subgrid.width - src_start_col,
                           GRID_NUM_COLS - dest_start_col);
        for row in 0..num_rows {
            for col in 0..num_cols {
                let tile = &subgrid[(src_start_col + col,
                                     src_start_row + row)];
                if tile.is_some() {
                    self[(dest_start_col + col, dest_start_row + row)] =
                        tile.clone();
                }
            }
        }
    }

    pub fn save<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let (red, green, blue) = self.background_color;
        try!(write!(writer, "@BG {} {} {}\n", red, green, blue));
        try!(write!(writer, "{}\n", self.tileset.dirpath().display()));
        for filename in self.tileset.filenames() {
            try!(write!(writer, ">{}\n", filename));
        }
        try!(write!(writer, "\n"));
        let mut map = BTreeMap::<String, usize>::new();
        for (index, filename) in self.tileset.filenames().enumerate() {
            map.insert(filename.clone(), index);
        }
        for row in 0..GRID_NUM_ROWS {
            let mut spaces = 0;
            for col in 0..GRID_NUM_COLS {
                match self[(col, row)] {
                    Some(ref tile) => {
                        for _ in 0..spaces {
                            try!(write!(writer, "  "));
                        }
                        spaces = 0;
                        let file_index = *map.get(&tile.filename).unwrap();
                        let char1 = index_to_base62(file_index);
                        let char2 = index_to_base62(tile.index);
                        try!(write!(writer, "{}{}", char1, char2));
                    }
                    None => {
                        spaces += 1;
                    }
                }
            }
            try!(write!(writer, "\n"));
        }
        Ok(())
    }

    pub fn load<R: io::Read>(canvas: &Canvas,
                             mut reader: R)
                             -> io::Result<TileGrid> {
        try!(read_exactly(reader.by_ref(), b"@BG "));
        let red = try!(read_int(reader.by_ref(), b' ')) as u8;
        let green = try!(read_int(reader.by_ref(), b' ')) as u8;
        let blue = try!(read_int(reader.by_ref(), b'\n')) as u8;
        let dirpath = PathBuf::from(try!(read_string(reader.by_ref(), b'\n')));
        let mut filenames = Vec::new();
        loop {
            match try!(read_byte(reader.by_ref())) {
                b'>' => {
                    filenames.push(try!(read_string(reader.by_ref(), b'\n')));
                }
                b'\n' => break,
                byte => {
                    let msg = format!("unexpected byte: {}", byte);
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                                              msg));
                }
            }
        }
        let tileset = try!(Tileset::load(canvas, dirpath, &filenames));
        let mut grid = Vec::new();
        for _ in 0..GRID_NUM_ROWS {
            let mut col = 0;
            loop {
                let byte1 = try!(read_byte(reader.by_ref()));
                if byte1 == b'\n' {
                    for _ in col..GRID_NUM_COLS {
                        grid.push(None);
                    }
                    break;
                }
                if col >= GRID_NUM_COLS {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                                              "too many columns"));
                }
                let byte2 = try!(read_byte(reader.by_ref()));
                if byte1 == b' ' && byte2 == b' ' {
                    grid.push(None);
                } else {
                    let file_index = try!(base62_to_index(byte1));
                    let tile_index = try!(base62_to_index(byte2));
                    let opt_tile = tileset.get(file_index, tile_index);
                    let tile = try!(opt_tile.ok_or_else(|| {
                        let msg = format!("invalid tile: {} {}", byte1, byte2);
                        io::Error::new(io::ErrorKind::InvalidData, msg)
                    }));
                    grid.push(Some(tile));
                }
                col += 1;
            }
        }
        Ok(TileGrid {
            background_color: (red, green, blue),
            tileset: Rc::new(tileset),
            grid: grid,
        })
    }

    pub fn load_from_path(canvas: &Canvas,
                          path: &String)
                          -> io::Result<TileGrid> {
        TileGrid::load(canvas, try!(File::open(path)))
    }
}

fn index_to_base62(index: usize) -> char {
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
     'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
     'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
     'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3',
     '4', '5', '6', '7', '8', '9'][index]
}

fn base62_to_index(byte: u8) -> io::Result<usize> {
    match byte {
        b'A'...b'Z' => Ok((byte - b'A') as usize),
        b'a'...b'z' => Ok((byte - b'a') as usize + 26),
        b'0'...b'9' => Ok((byte - b'0') as usize + 52),
        _ => {
            let msg = format!("invalid index byte: {}", byte);
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    }
}

impl Index<(u32, u32)> for TileGrid {
    type Output = Option<Tile>;
    fn index(&self, (col, row): (u32, u32)) -> &Option<Tile> {
        if col >= GRID_NUM_COLS || row >= GRID_NUM_ROWS {
            panic!("index out of range");
        }
        &self.grid[(row * GRID_NUM_COLS + col) as usize]
    }
}

impl IndexMut<(u32, u32)> for TileGrid {
    fn index_mut(&mut self, (col, row): (u32, u32)) -> &mut Option<Tile> {
        if col >= GRID_NUM_COLS || row >= GRID_NUM_ROWS {
            panic!("index out of range");
        }
        &mut self.grid[(row * GRID_NUM_COLS + col) as usize]
    }
}

// ========================================================================= //

fn read_byte<R: io::Read>(reader: R) -> io::Result<u8> {
    match reader.bytes().next() {
        Some(result) => result,
        None => {
            let msg = "unexpected EOF";
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    }
}

fn read_exactly<R: io::Read>(mut reader: R, string: &[u8]) -> io::Result<()> {
    let mut actual = vec![0u8; string.len()];
    try!(reader.read_exact(&mut actual));
    if &actual as &[u8] != string {
        let msg = format!("expected '{}', found '{}'",
                          String::from_utf8_lossy(string),
                          String::from_utf8_lossy(&actual));
        Err(io::Error::new(io::ErrorKind::InvalidData, msg))
    } else {
        Ok(())
    }
}

fn read_int<R: io::Read>(reader: R, terminator: u8) -> io::Result<u32> {
    let mut value: u32 = 0;
    for next in reader.bytes() {
        let byte = try!(next);
        if byte == terminator {
            break;
        }
        let digit: u8;
        if b'0' <= byte && byte <= b'9' {
            digit = byte - b'0';
        } else {
            let msg = format!("invalid character in header field: '{}'",
                              String::from_utf8_lossy(&[byte]));
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
        value = value * 10 + digit as u32;
        if value > 0xFFFF {
            let msg = "value is too large";
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
    }
    Ok(value)
}

fn read_string<R: io::Read>(reader: R, terminator: u8) -> io::Result<String> {
    let mut result = Vec::new();
    for next in reader.bytes() {
        let byte = try!(next);
        if byte == terminator {
            break;
        }
        result.push(byte);
    }
    String::from_utf8(result).map_err(|_| {
        let msg = "invalid utf8";
        io::Error::new(io::ErrorKind::InvalidData, msg)
    })
}

// ========================================================================= //
