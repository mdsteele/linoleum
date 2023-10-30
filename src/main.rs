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

extern crate ahi;
extern crate getopts;
extern crate sdl2;

mod canvas;
mod coords;
mod editor;
mod element;
mod event;
mod paint;
mod palette;
mod state;
mod textbox;
mod tilegrid;
mod toolbox;
mod unsaved;
mod util;

use self::canvas::{Font, Sprite, Window};
use self::editor::EditorView;
use self::element::GuiElement;
use self::event::Event;
use self::state::EditorState;
use self::tilegrid::{TileGrid, Tileset};
use ahi::Palette;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

//===========================================================================//

const FRAME_DELAY_MILLIS: u32 = 100;

fn render_screen(window: &mut Window, state: &EditorState, gui: &EditorView) {
    {
        let mut canvas = window.canvas();
        canvas.clear((64, 64, 64, 255));
        gui.draw(state, &mut canvas);
    }
    window.present();
}

fn load_font(window: &Window, path: &str) -> Font {
    let ahf = util::load_ahf_from_file(&path.to_string()).unwrap();
    window.new_font(&ahf)
}

fn load_sprite(window: &Window, path: &str) -> Sprite {
    let collection = util::load_ahi_from_file(&path.to_string()).unwrap();
    let palette = collection.palettes.first().unwrap_or(Palette::default());
    window.new_sprite(&collection.images[0], palette)
}

fn load_sprites(window: &Window, path: &str) -> Vec<Sprite> {
    let collection = util::load_ahi_from_file(&path.to_string()).unwrap();
    let palette = collection.palettes.first().unwrap_or(Palette::default());
    collection
        .images
        .iter()
        .map(|image| window.new_sprite(image, palette))
        .collect()
}

//===========================================================================//

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("", "tiles", "set tiles directory", "DIR");
    opts.optopt("", "bg", "background file to open", "FILE");
    let matches = opts.parse(&args[1..]).unwrap_or_else(|failure| {
        println!("Error: {:?}", failure);
        println!("Run with --help to see available flags.");
        std::process::exit(1);
    });
    if matches.opt_present("help") {
        let brief = format!("Usage: {} [options]", &args[0]);
        print!("{}", opts.usage(&brief));
        std::process::exit(0);
    }
    let tiles_dir =
        PathBuf::from(matches.opt_str("tiles").unwrap_or("tiles".to_string()));

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window_width = 720;
    let window_height = 440;
    let sdl_window = video_subsystem
        .window("Linoleum", window_width * 2, window_height * 2)
        .position_centered()
        //.fullscreen_desktop()
        .build()
        .unwrap();
    let mut renderer = sdl_window.into_canvas().build().unwrap();
    renderer.set_logical_size(window_width, window_height).unwrap();
    let mut window = Window::from_renderer(&mut renderer);

    let tool_icons: Vec<Sprite> = load_sprites(&window, "data/tool_icons.ahi");
    let arrow_icons: Vec<Sprite> = load_sprites(&window, "data/arrows.ahi");
    let unsaved_icon = load_sprite(&window, "data/unsaved.ahi");
    let font: Rc<Font> = Rc::new(load_font(&window, "data/font.ahf"));

    let mut state = if let Some(path) = matches.opt_str("bg") {
        match TileGrid::load_from_path(&window, &tiles_dir, &path) {
            Ok(tilegrid) => EditorState::new(path, tilegrid),
            Err(err) => {
                println!("Failed to load bg: {:?}", err);
                std::process::exit(0);
            }
        }
    } else {
        let tileset =
            Tileset::load(&window, &tiles_dir, &["green_pipes".to_string()])
                .unwrap();
        EditorState::new("out.bg".to_string(), TileGrid::new(tileset))
    };

    let mut gui = EditorView::new(tool_icons, arrow_icons, unsaved_icon, font);
    render_screen(&mut window, &state, &gui);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_clock_tick = Instant::now();
    loop {
        let now = Instant::now();
        let elapsed_millis = now
            .duration_since(last_clock_tick)
            .as_millis()
            .min(u32::MAX as u128) as u32;
        let opt_sdl_event = if elapsed_millis >= FRAME_DELAY_MILLIS {
            None
        } else {
            event_pump.wait_event_timeout(FRAME_DELAY_MILLIS - elapsed_millis)
        };
        let event = match opt_sdl_event {
            None => {
                last_clock_tick = now;
                Event::ClockTick
            }
            Some(sdl_event) => match Event::from_sdl2(&sdl_event) {
                Some(event) => event,
                None => continue,
            },
        };
        let mut action = match event {
            Event::Quit => return,
            event => gui.on_event(&event, &mut state),
        };
        if let Some((mode, text)) = action.take_value() {
            if gui.mode_perform(&window, &mut state, mode, text) {
                action.also_redraw();
            }
        }
        if action.should_redraw() {
            render_screen(&mut window, &state, &gui);
        }
    }
}

//===========================================================================//
