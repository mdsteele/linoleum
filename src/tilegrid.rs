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

use super::canvas::{Sprite, Window};
use super::util;
use ahi::Palette;
use sdl2::rect::{Point, Rect};
use std::cmp::{max, min, Ordering};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io;
use std::ops::{Deref, Index, IndexMut};
use std::path::{Path, PathBuf};
use std::rc::Rc;

//===========================================================================//

const DEFAULT_TILE_SIZE: u32 = 8;

#[derive(Clone)]
pub struct Tileset {
    dirpath: PathBuf,
    tiles: Vec<(String, Vec<Rc<Sprite>>)>,
    tile_size: u32,
}

impl Tileset {
    pub fn load(
        window: &Window,
        dirpath: &Path,
        filenames: &[String],
    ) -> io::Result<Tileset> {
        let mut tiles = vec![];
        for filename in filenames {
            let path = dirpath.join(filename).with_extension("ahi");
            let collection =
                util::load_ahi_from_file(&path.to_str().unwrap().to_string())?;
            let palette =
                collection.palettes.first().unwrap_or(Palette::default());
            let mut sprites = vec![];
            for image in collection.images {
                let sprite = window.new_sprite(&image, palette);
                sprites.push(Rc::new(sprite));
            }
            tiles.push((filename.to_string(), sprites));
        }
        let tile_size = Tileset::max_tile_size(&tiles);
        Ok(Tileset { dirpath: dirpath.to_path_buf(), tiles, tile_size })
    }

    pub fn reload(
        &mut self,
        window: &Window,
        filenames: &[&str],
    ) -> io::Result<()> {
        let mut old_tiles: BTreeMap<String, Vec<Rc<Sprite>>> = BTreeMap::new();
        for &(ref filename, ref sprites) in self.tiles.iter() {
            old_tiles.insert(filename.clone(), sprites.clone());
        }
        let mut new_tiles: Vec<(String, Vec<Rc<Sprite>>)> = Vec::new();
        for filename in filenames {
            if let Some(sprites) = old_tiles.get(&filename.to_string()) {
                new_tiles.push((filename.to_string(), sprites.clone()));
                continue;
            }
            let path = self.dirpath.join(filename).with_extension("ahi");
            let collection =
                util::load_ahi_from_file(&path.to_str().unwrap().to_string())?;
            let palette =
                collection.palettes.first().unwrap_or(Palette::default());
            let mut sprites = vec![];
            for image in collection.images {
                let sprite = window.new_sprite(&image, &palette);
                sprites.push(Rc::new(sprite));
            }
            new_tiles.push((filename.to_string(), sprites));
        }
        self.tiles = new_tiles;
        self.tile_size = Tileset::max_tile_size(&self.tiles);
        Ok(())
    }

    pub fn dirpath(&self) -> &Path {
        &self.dirpath
    }

    pub fn num_filenames(&self) -> usize {
        self.tiles.len()
    }

    pub fn filenames(&self) -> Filenames {
        Filenames { tileset: self, index: 0 }
    }

    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    pub fn tiles(&self, file_index: usize) -> Tiles {
        Tiles { tileset: self, file_index, tile_index: 0 }
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

    pub fn max_tile_size(tiles: &Vec<(String, Vec<Rc<Sprite>>)>) -> u32 {
        let mut max = 0;
        for &(_, ref sprites) in tiles.iter() {
            for sprite in sprites.iter() {
                let size = sprite.width().max(sprite.height());
                max = max.max(size);
            }
        }
        if max == 0 {
            max = DEFAULT_TILE_SIZE;
        }
        max
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

//===========================================================================//

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

//===========================================================================//

#[derive(Clone)]
pub struct SubGrid {
    width: u32,
    height: u32,
    grid: Vec<Option<Tile>>,
}

impl SubGrid {
    pub fn new(width: u32, height: u32) -> SubGrid {
        SubGrid { width, height, grid: vec![None; (width * height) as usize] }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn flip_horz(&mut self) {
        let mut new_grid: Vec<Option<Tile>> = vec![None; self.grid.len()];
        for row in 0..self.height {
            for col in 0..self.width {
                new_grid
                    [(row * self.width + (self.width - col - 1)) as usize] =
                    self[(col, row)].clone();
            }
        }
        self.grid = new_grid;
    }

    pub fn flip_vert(&mut self) {
        let mut new_grid: Vec<Option<Tile>> = vec![None; self.grid.len()];
        for row in 0..self.height {
            for col in 0..self.width {
                new_grid
                    [((self.height - row - 1) * self.width + col) as usize] =
                    self[(col, row)].clone();
            }
        }
        self.grid = new_grid;
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

impl IndexMut<(u32, u32)> for SubGrid {
    fn index_mut(&mut self, (col, row): (u32, u32)) -> &mut Option<Tile> {
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &mut self.grid[(row * self.width + col) as usize]
    }
}

//===========================================================================//

pub const GRID_DEFAULT_NUM_COLS: u32 = 36;
pub const GRID_DEFAULT_NUM_ROWS: u32 = 24;

#[derive(Clone)]
pub struct TileGrid {
    background_color: (u8, u8, u8),
    tileset: Rc<Tileset>,
    subgrid: SubGrid,
}

impl TileGrid {
    pub fn new(tileset: Tileset) -> TileGrid {
        TileGrid {
            background_color: (15, 15, 15),
            tileset: Rc::new(tileset),
            subgrid: SubGrid::new(
                GRID_DEFAULT_NUM_COLS,
                GRID_DEFAULT_NUM_ROWS,
            ),
        }
    }

    pub fn width(&self) -> u32 {
        self.subgrid.width()
    }

    pub fn height(&self) -> u32 {
        self.subgrid.height()
    }

    pub fn size(&self) -> (u32, u32) {
        self.subgrid.size()
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let mut new_subgrid = SubGrid::new(new_width, new_height);
        for row in 0..self.height().min(new_height) {
            for col in 0..self.width().min(new_width) {
                new_subgrid[(col, row)] = self.subgrid[(col, row)].take();
            }
        }
        self.subgrid = new_subgrid;
    }

    pub fn tile_size(&self) -> u32 {
        self.tileset.tile_size()
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

    pub fn set_tile_filenames(
        &mut self,
        window: &Window,
        filenames: Vec<&str>,
    ) -> io::Result<()> {
        Rc::make_mut(&mut self.tileset).reload(window, &filenames)?;
        let filenames_set: BTreeSet<String> =
            filenames.iter().cloned().map(str::to_string).collect();
        for tile in self.subgrid.grid.iter_mut() {
            let bad = match *tile {
                Some(ref tile) => !filenames_set.contains(&tile.filename),
                None => false,
            };
            if bad {
                *tile = None;
            }
        }
        Ok(())
    }

    pub fn copy_subgrid(&self, rect: Rect) -> SubGrid {
        let mut grid = Vec::new();
        let start_col = max(0, rect.left()) as u32;
        let end_col = min(self.width() as i32, rect.right()) as u32;
        let start_row = max(0, rect.top()) as u32;
        let end_row = min(self.height() as i32, rect.bottom()) as u32;
        for row in start_row..end_row {
            for col in start_col..end_col {
                grid.push(self[(col, row)].clone());
            }
        }
        SubGrid {
            width: end_col - start_col,
            height: end_row - start_row,
            grid,
        }
    }

    pub fn cut_subgrid(&mut self, rect: Rect) -> SubGrid {
        let mut grid = Vec::new();
        let start_col = max(0, rect.left()) as u32;
        let end_col = min(self.width() as i32, rect.right()) as u32;
        let start_row = max(0, rect.top()) as u32;
        let end_row = min(self.height() as i32, rect.bottom()) as u32;
        for row in start_row..end_row {
            for col in start_col..end_col {
                grid.push(self[(col, row)].clone());
                self[(col, row)] = None;
            }
        }
        SubGrid {
            width: end_col - start_col,
            height: end_row - start_row,
            grid,
        }
    }

    pub fn paste_subgrid(&mut self, subgrid: &SubGrid, topleft: Point) {
        let src_start_row = min(max(0, -topleft.y()) as u32, subgrid.height);
        let src_start_col = min(max(0, -topleft.x()) as u32, subgrid.width);
        let dest_start_row = min(max(0, topleft.y()) as u32, self.height());
        let dest_start_col = min(max(0, topleft.x()) as u32, self.width());
        let num_rows = min(
            subgrid.height - src_start_row,
            self.height() - dest_start_row,
        );
        let num_cols =
            min(subgrid.width - src_start_col, self.width() - dest_start_col);
        for row in 0..num_rows {
            for col in 0..num_cols {
                let tile =
                    &subgrid[(src_start_col + col, src_start_row + row)];
                if tile.is_some() {
                    self[(dest_start_col + col, dest_start_row + row)] =
                        tile.clone();
                }
            }
        }
    }

    pub fn save<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let (red, green, blue) = self.background_color;
        write!(writer, "@BG {} {} {}", red, green, blue)?;
        if self.width() == GRID_DEFAULT_NUM_COLS
            && self.height() == GRID_DEFAULT_NUM_ROWS
        {
            write!(writer, "\n")?;
        } else {
            write!(writer, " {}x{}\n", self.width(), self.height())?;
        }
        for filename in self.tileset.filenames() {
            write!(writer, ">{}\n", filename)?;
        }
        let mut map = BTreeMap::<String, usize>::new();
        for (index, filename) in self.tileset.filenames().enumerate() {
            map.insert(filename.clone(), index);
        }
        let mut lines = Vec::<String>::new();
        for row in 0..self.height() {
            let mut line = String::new();
            let mut spaces = 0;
            for col in 0..self.width() {
                match self[(col, row)] {
                    Some(ref tile) => {
                        for _ in 0..spaces {
                            line.push_str("  ");
                        }
                        spaces = 0;
                        let file_index = *map.get(&tile.filename).unwrap();
                        let char1 = index_to_base64(file_index);
                        let char2 = index_to_base64(tile.index);
                        line.push_str(&format!("{}{}", char1, char2));
                    }
                    None => {
                        spaces += 1;
                    }
                }
            }
            lines.push(line);
        }
        while matches!(lines.last().map(String::deref), Some("")) {
            lines.pop();
        }
        if !lines.is_empty() {
            write!(writer, "\n")?;
            for line in lines {
                writeln!(writer, "{}", line)?;
            }
        }
        Ok(())
    }

    pub fn load<R: io::Read>(
        window: &Window,
        dirpath: &Path,
        mut reader: R,
    ) -> io::Result<TileGrid> {
        read_exactly(reader.by_ref(), b"@BG ")?;
        let red = read_int_with(reader.by_ref(), b' ')?;
        let green = read_int_with(reader.by_ref(), b' ')?;
        let (blue, next) = read_int(reader.by_ref())?;
        let (width, height) = if next == b'\n' {
            (GRID_DEFAULT_NUM_COLS, GRID_DEFAULT_NUM_ROWS)
        } else if next == b' ' {
            let width = read_int_with(reader.by_ref(), b'x')?;
            let height = read_int_with(reader.by_ref(), b'\n')?;
            (width, height)
        } else {
            let msg = format!(
                "unexpected char '{}' in header",
                String::from_utf8_lossy(&[next])
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        };
        let background_color = (red as u8, green as u8, blue as u8);
        let mut subgrid = SubGrid::new(width, height);
        let mut filenames = Vec::new();
        loop {
            match read_byte_or_eof(reader.by_ref())? {
                Some(b'>') => {
                    filenames.push(read_string(reader.by_ref(), b'\n')?);
                }
                Some(b'\n') => break,
                Some(byte) => {
                    let msg = format!("unexpected byte: {}", byte);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        msg,
                    ));
                }
                None => {
                    let tileset =
                        Rc::new(Tileset::load(window, dirpath, &filenames)?);
                    return Ok(TileGrid {
                        background_color,
                        tileset,
                        subgrid,
                    });
                }
            }
        }
        let tileset = Rc::new(Tileset::load(window, dirpath, &filenames)?);
        for row in 0..height {
            let mut col = 0;
            loop {
                let byte1 = match read_byte_or_eof(reader.by_ref())? {
                    None => {
                        return Ok(TileGrid {
                            background_color,
                            tileset,
                            subgrid,
                        });
                    }
                    Some(b'\n') => break,
                    Some(byte) => byte,
                };
                if col >= width {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "too many columns",
                    ));
                }
                let byte2 = read_byte(reader.by_ref())?;
                if byte1 != b' ' || byte2 != b' ' {
                    let file_index = base64_to_index(byte1)?;
                    let tile_index = base64_to_index(byte2)?;
                    let opt_tile = tileset.get(file_index, tile_index);
                    let tile = opt_tile.ok_or_else(|| {
                        let msg = format!("invalid tile: {} {}", byte1, byte2);
                        io::Error::new(io::ErrorKind::InvalidData, msg)
                    })?;
                    subgrid[(col, row)] = Some(tile);
                }
                col += 1;
            }
        }
        return Ok(TileGrid { background_color, tileset, subgrid });
    }

    pub fn load_from_path(
        window: &Window,
        dirpath: &Path,
        path: &String,
    ) -> io::Result<TileGrid> {
        TileGrid::load(window, dirpath, File::open(path)?)
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn index_to_base64(index: usize) -> char {
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
     'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
     'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
     'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3',
     '4', '5', '6', '7', '8', '9', '+', '/'][index]
}

fn base64_to_index(byte: u8) -> io::Result<usize> {
    match byte {
        b'A'..=b'Z' => Ok((byte - b'A') as usize),
        b'a'..=b'z' => Ok((byte - b'a') as usize + 26),
        b'0'..=b'9' => Ok((byte - b'0') as usize + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => {
            let msg = format!("invalid index byte: {}", byte);
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    }
}

impl Index<(u32, u32)> for TileGrid {
    type Output = Option<Tile>;
    fn index(&self, (col, row): (u32, u32)) -> &Option<Tile> {
        &self.subgrid[(col, row)]
    }
}

impl IndexMut<(u32, u32)> for TileGrid {
    fn index_mut(&mut self, (col, row): (u32, u32)) -> &mut Option<Tile> {
        &mut self.subgrid[(col, row)]
    }
}

//===========================================================================//

fn read_byte_or_eof<R: io::Read>(reader: R) -> io::Result<Option<u8>> {
    match reader.bytes().next() {
        Some(result) => result.map(Option::Some),
        None => Ok(None),
    }
}

fn read_byte<R: io::Read>(reader: R) -> io::Result<u8> {
    match read_byte_or_eof(reader) {
        Err(error) => Err(error),
        Ok(Some(byte)) => Ok(byte),
        Ok(None) => {
            let msg = "unexpected EOF";
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    }
}

fn read_exactly<R: io::Read>(mut reader: R, string: &[u8]) -> io::Result<()> {
    let mut actual = vec![0u8; string.len()];
    reader.read_exact(&mut actual)?;
    if &actual as &[u8] != string {
        let msg = format!(
            "expected '{}', found '{}'",
            String::from_utf8_lossy(string),
            String::from_utf8_lossy(&actual)
        );
        Err(io::Error::new(io::ErrorKind::InvalidData, msg))
    } else {
        Ok(())
    }
}

fn read_int_with<R: io::Read>(reader: R, terminator: u8) -> io::Result<u32> {
    let (value, next) = read_int(reader)?;
    if next != terminator {
        let msg = format!(
            "expected '{}' in header but found '{}'",
            String::from_utf8_lossy(&[terminator]),
            String::from_utf8_lossy(&[next])
        );
        return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
    }
    Ok(value)
}

fn read_int<R: io::Read>(reader: R) -> io::Result<(u32, u8)> {
    let mut value: u32 = 0;
    for next in reader.bytes() {
        let byte = next?;
        let digit: u8;
        if b'0' <= byte && byte <= b'9' {
            digit = byte - b'0';
        } else {
            return Ok((value, byte));
        }
        value = value * 10 + digit as u32;
        if value > 0xFF {
            let msg = "value is too large";
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
    }
    Ok((value, 0))
}

fn read_string<R: io::Read>(reader: R, terminator: u8) -> io::Result<String> {
    let mut result = Vec::new();
    for next in reader.bytes() {
        let byte = next?;
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

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{base64_to_index, index_to_base64};

    #[test]
    fn base64_round_trip() {
        for index in 0..64 {
            let ch = index_to_base64(index);
            let i: u32 = ch.into();
            assert!(i <= (u8::MAX as u32));
            assert_eq!(Some(index), base64_to_index(i as u8).ok());
        }
    }
}

//===========================================================================//
