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

use ahi;
use std::fs::File;
use std::io;

// ========================================================================= //

pub fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!();
    }
    let remainder = a % b;
    if remainder == 0 {
        0
    } else if (a < 0) ^ (b < 0) {
        remainder + b
    } else {
        remainder
    }
}

// ========================================================================= //

pub fn load_ahf_from_file(path: &String) -> io::Result<ahi::Font> {
    let mut file = try!(File::open(path));
    ahi::Font::read(&mut file)
}

pub fn load_ahi_from_file(path: &String) -> io::Result<Vec<ahi::Image>> {
    let mut file = try!(File::open(path));
    ahi::Image::read_all(&mut file)
}

// ========================================================================= //
