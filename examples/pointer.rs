#[macro_use]
extern crate wlroots;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wlroots::compositor::Compositor;
use wlroots::cursor::{Cursor, XCursor, XCursorTheme};
use wlroots::device::Device;
use wlroots::key_event::KeyEvent;
use wlroots::manager::{InputManagerHandler, OutputManagerHandler};
use wlroots::output::{Output, OutputLayout};
use wlroots::pointer;
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::{WLR_BUTTON_PRESSED, WLR_BUTTON_RELEASED};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct OutputHandler {
    pub layout: OutputLayout,
    color: Rc<Cell<[f32; 4]>>,
    cursor: Rc<RefCell<Cursor>>
}

struct InputHandler {
    color: Rc<Cell<[f32; 4]>>,
    default_color: [f32; 4],
    cursor: Rc<RefCell<Cursor>>
}

impl OutputManagerHandler for OutputHandler {
    fn output_added(&mut self, mut output: Output) {
        let mut cursor = self.cursor.borrow_mut();
        {
            let xcursor = cursor.xcursor().expect("XCursor was not set!");
            // TODO Remove
            //let (mut cursor, mut xcursor) = segfault();
            let image = &xcursor.images()[0];
            // TODO use output config if present instead of auto
            self.layout.add_auto(&mut output);
            if output.set_cursor(image).is_err() {
                wlr_log!(L_DEBUG, "Failed to set hardware cursor");
                return
            }
        }
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
    }
}

impl InputManagerHandler for InputHandler {
    fn button(&mut self, event: pointer::ButtonEvent) {
        if event.state() == WLR_BUTTON_RELEASED {
            self.color.set(self.default_color.clone())
        } else {
            let mut red: [f32; 4] = [0.25, 0.25, 0.25, 1.0];
            red[event.button() as usize % 3] = 1.0;
            self.color.set(red);
        }
    }
}

// FIXME
// Ok, so this causes a segfault when you uncomment the line in output_added.
// The _main_ reason is that we drop the layout, as that unsafe line entails.
// _HOWEVER_, we also drop the xcursor theme, and we can also destroy the cursor
// (in fact both the xcursor and cursor are dropped in output_added once this func
// is called).
//
// SO We need to make sure _all_ of those things can't happen, otherwise it's UB
fn segfault() -> (Cursor, XCursor) {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut xcursor = xcursor_theme
        .get_cursor("left_ptr".into())
        .expect("Could not load cursor from theme");
    let mut layout = OutputLayout::new();
    // TODO Ensure this can be safe...
    // e.g what's stopping me from simply dropping layout now that I gave it to
    // cursor?
    unsafe {
        cursor.attach_output_layout(&mut layout);
    }
    (cursor, xcursor)
}

fn managers(mut cursor: Cursor) -> (OutputHandler, InputHandler) {
    let mut layout = OutputLayout::new();
    // TODO Ensure this can be safe...
    // e.g what's stopping me from simply dropping layout now that I gave it to
    // cursor?
    unsafe {
        cursor.attach_output_layout(&mut layout);
    }
    let cursor = Rc::new(RefCell::new(cursor));
    let color = Rc::new(Cell::new([0.25, 0.25, 0.25, 1.0]));
    (OutputHandler {
        color: color.clone(),
        layout,
        cursor: cursor.clone()
     },
     InputHandler {
         color: color.clone(),
         default_color: color.get(),
         cursor: cursor.clone()
     })
}

fn main() {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut xcursor = xcursor_theme
        .get_cursor("left_ptr".into())
        .expect("Could not load cursor from theme");
    cursor.set_xcursor(Some(xcursor));

    let (mut output_manager, input_manager) = managers(cursor);
    let compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));
    compositor.run();
}
