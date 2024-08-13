use sdl2::{self, render::Canvas, video::Window, AudioSubsystem, EventPump};

pub struct Screen {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub audio: AudioSubsystem
}

impl Screen {
    pub fn new(width: Option<u32>, height: Option<u32>) -> Screen {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let window = video_subsystem
            .window("SDL2", width.unwrap_or(640), height.unwrap_or(480))
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
            event_pump,
            audio: audio_subsystem
        }
    }
}
