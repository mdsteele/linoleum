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
extern crate sdl2;

mod canvas;
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
use self::element::{Action, AggregateElement, GuiElement};
use self::event::{COMMAND, Event, Keycode, SHIFT};
use self::paint::GridCanvas;
use self::palette::TilePalette;
use self::state::EditorState;
use self::textbox::ModalTextBox;
use self::tilegrid::Tileset;
use self::toolbox::Toolbox;
use self::unsaved::UnsavedIndicator;
use std::path::PathBuf;
use std::rc::Rc;

// ========================================================================= //

const FRAME_DELAY_MILLIS: u32 = 100;

fn render_screen<E: GuiElement<EditorState>>(window: &mut Window,
                                             state: &EditorState,
                                             gui: &E) {
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
    let images = util::load_ahi_from_file(&path.to_string()).unwrap();
    window.new_sprite(&images[0])
}

fn load_sprites(window: &Window, path: &str) -> Vec<Sprite> {
    let images = util::load_ahi_from_file(&path.to_string()).unwrap();
    images.iter().map(|image| window.new_sprite(image)).collect()
}

// ========================================================================= //

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let event_subsystem = sdl_context.event().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window_width = 720;
    let window_height = 450;
    let sdl_window = video_subsystem.window("Linoleum",
                                            window_width,
                                            window_height)
                                    .position_centered()
                                    .fullscreen_desktop()
                                    .build()
                                    .unwrap();
    let mut renderer = sdl_window.renderer().build().unwrap();
    renderer.set_logical_size(window_width, window_height).unwrap();
    let mut window = Window::from_renderer(&mut renderer);

    let tool_icons: Vec<Sprite> = load_sprites(&window, "data/tool_icons.ahi");
    let arrow_icons: Vec<Sprite> = load_sprites(&window, "data/arrows.ahi");
    let unsaved_icon = load_sprite(&window, "data/unsaved.ahi");
    let font: Rc<Font> = Rc::new(load_font(&window, "data/font.ahf"));

    let tileset = Tileset::load(&window,
                                &PathBuf::from("tiles"),
                                &["blue_ells".to_string(),
                                  "green_pipes".to_string(),
                                  "red_brick".to_string(),
                                  "girders".to_string(),
                                  "caution_walls".to_string()])
                      .unwrap();
    let mut state = EditorState::new("out.bg".to_string(), tileset);


    let elements: Vec<Box<GuiElement<EditorState>>> = vec![
        Box::new(ModalTextBox::new(10, 420, font.clone())),
        Box::new(Toolbox::new(10, 10, tool_icons)),
        Box::new(TilePalette::new(10, 72, arrow_icons)),
        Box::new(GridCanvas::new(72, 10)),
        Box::new(UnsavedIndicator::new(694, 10, unsaved_icon)),
    ];
    let mut gui = AggregateElement::new(elements);

    render_screen(&mut window, &state, &gui);

    Event::register_clock_ticks(&event_subsystem);
    let _timer =
        timer_subsystem.add_timer(FRAME_DELAY_MILLIS,
                                  Box::new(|| {
                                      Event::push_clock_tick(&event_subsystem);
                                      FRAME_DELAY_MILLIS
                                  }));

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        let event = match Event::from_sdl2(&event_pump.wait_event()) {
            Some(event) => event,
            None => continue,
        };
        let action = match event {
            Event::Quit => return,
            Event::KeyDown(Keycode::A, kmod) if kmod == COMMAND => {
                state.mutation().select_all();
                Action::redraw().and_stop()
            }
            Event::KeyDown(Keycode::B, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_change_color()).and_stop()
            }
            Event::KeyDown(Keycode::C, kmod) if kmod == COMMAND => {
                state.mutation().copy_selection();
                Action::ignore().and_stop()
            }
            Event::KeyDown(Keycode::O, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_load_file()).and_stop()
            }
            Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND => {
                state.save_to_file().unwrap();
                Action::redraw().and_stop()
            }
            Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.begin_save_as()).and_stop()
            }
            Event::KeyDown(Keycode::T, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_change_tiles()).and_stop()
            }
            Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND => {
                state.mutation().paste_selection();
                Action::redraw().and_stop()
            }
            Event::KeyDown(Keycode::X, kmod) if kmod == COMMAND => {
                state.mutation().cut_selection();
                Action::redraw().and_stop()
            }
            Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.undo()).and_stop()
            }
            Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.redo()).and_stop()
            }
            event => gui.handle_event(&event, &mut state),
        };
        if state.mode_perform_if_necessary(&window) || action.should_redraw() {
            render_screen(&mut window, &state, &gui);
        }
    }
}

// ========================================================================= //
