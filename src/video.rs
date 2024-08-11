use sdl2::{self, render::Canvas, video::Window, EventPump};

pub struct Screen {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump
}

impl Screen {
    pub fn new() -> Screen {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("SDL2", 640, 480)
            .position_centered()
            .build()
            .map_err(|e| e.to_string()).unwrap();

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| e.to_string()).unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        Screen {
            canvas,
            event_pump
        }
    }
}
