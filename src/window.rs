use std::sync::{Arc, Mutex};
use std::any::Any;

use glutin::dpi::PhysicalSize;
use glutin::{ContextBuilder, ControlFlow, Event, EventsLoop, GlProfile, GlRequest, KeyboardInput};
use glutin::{VirtualKeyCode, WindowBuilder, WindowEvent};
use pathfinder_canvas::{CanvasFontContext, CanvasRenderingContext2D, Path2D};
use pathfinder_content::color::{ColorF, ColorU};
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use pathfinder_geometry::rect::RectF;
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_gpu::resources::FilesystemResourceLoader;
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererOptions};
use pathfinder_renderer::options::BuildOptions;

use neovim_lib::NeovimApi;

use crate::editor::{DrawCommand, Editor, Colors};

// #[derive(new)]
// struct WindowState {
//     editor: Arc<Mutex<Editor>>,

//     #[new(default)]
//     pub size: (f64, f64),
//     #[new(default)]
//     pub handle: WindowHandle,
//     #[new(default)]
//     pub font: Option<PietFont>
// }

// const FONT_NAME: &str = "Delugia Nerd Font";
// const FONT_SIZE: f64 = 14.0;
// const FONT_WIDTH: f64 = 8.2;
// const FONT_HEIGHT: f64 = 16.4;

// fn process_draw_commands(draw_commands: &Vec<DrawCommand>, default_colors: &Colors, piet: &mut Piet, font: &PietFont) {
//     for command in draw_commands {
//         let x = command.col_start as f64 * FONT_WIDTH;
//         let y = command.row as f64 * FONT_HEIGHT + FONT_HEIGHT;
//         let top = y - FONT_HEIGHT * 0.8;
//         let top_left = (x, top);
//         let width = command.text.chars().count() as f64 * FONT_WIDTH;
//         let height = FONT_HEIGHT;
//         let bottom_right = (x + width, top + height);
//         let region = Rect::from_points(top_left, bottom_right);
//         piet.fill(region, &command.style.colors.background.clone().or(default_colors.background.clone()).unwrap());
            
//         let piet_text = piet.text();
//         let text_layout = piet_text.new_text_layout(&font, &command.text).build().unwrap();
//         piet.draw_text(&text_layout, (x, y), &command.style.colors.foreground.clone().or(default_colors.foreground.clone()).unwrap());
//     }
// }

// impl WinHandler for WindowState {
//     fn connect(&mut self, handle: &WindowHandle) {
//         self.handle = handle.clone();
//     }

//     fn paint(&mut self, piet: &mut Piet, _ctx: &mut dyn WinCtx) -> bool {
//         let text = piet.text();
//         if self.font.is_none() {
//             self.font = Some(text.new_font_by_name(FONT_NAME, FONT_SIZE).build().unwrap());
//         }
//         let font = self.font.as_ref().unwrap();

//         let editor = self.editor.lock().unwrap();
//         let draw_commands = editor.build_draw_commands();

//         piet.clear(editor.default_colors.background.clone().unwrap());
//         process_draw_commands(&draw_commands, &editor.default_colors, piet, font);

//         let (cursor_grid_x, cursor_grid_y) = editor.cursor_pos;
//         let cursor_x = cursor_grid_x as f64 * FONT_WIDTH;
//         let cursor_width = FONT_WIDTH / 8.0;
//         let cursor_y = cursor_grid_y as f64 * FONT_HEIGHT + FONT_HEIGHT * 0.2;
//         let cursor_height = FONT_HEIGHT;
//         let cursor = Rect::from_points((cursor_x, cursor_y), (cursor_x + cursor_width, cursor_y + cursor_height));
//         piet.fill(cursor, &Color::rgb8(0xff, 0xff, 0xff));
//         true
//     }

//     fn key_down(&mut self, key_event: KeyEvent, _ctx: &mut dyn WinCtx) -> bool {
//         let mut editor = self.editor.lock().unwrap();
//         match key_event {
//             k_e if k_e.key_code.is_printable() => {
//                 let incoming_text = k_e.unmod_text().unwrap_or("");
//                 editor.nvim.input(incoming_text).expect("Input call failed...");
//             },
//             k_e if (HotKey::new(None, KeyCode::Escape)).matches(k_e) => {
//                 editor.nvim.input("<Esc>").expect("Input call failed...");
//             },
//             k_e if (HotKey::new(None, KeyCode::Backspace)).matches(k_e) => {
//                 editor.nvim.input("<BS>").expect("Input call failed...");
//             }
//             _ => ()
//         };
//         true
//     }

//     fn size(&mut self, width: u32, height: u32, _ctx: &mut dyn WinCtx) {
//         let dpi = self.handle.get_dpi();
//         let dpi_scale = dpi as f64 / 96.0;
//         let width_f = (width as f64) / dpi_scale;
//         let height_f = (height as f64) / dpi_scale;
//         self.size = (width_f, height_f);

//         let mut editor = self.editor.lock().unwrap();
//         editor.resize((width_f / FONT_WIDTH) as u16, (height_f / FONT_HEIGHT) as u16);
//     }

//     fn as_any(&mut self) -> &mut dyn Any {
//         self
//     }
// }

pub fn ui_loop(editor: Arc<Mutex<Editor>>) {
    let mut event_loop = EventsLoop::new();
    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor();

    let (width, height) = { editor.lock().unwrap().size };
    let window_size = Vector2I::new(width as i32, height as i32);

    let physical_window_size = PhysicalSize::new(width as f64, height as f64);
    let logical_window_size = physical_window_size.to_logical(hidpi_factor);
    let window_builder = WindowBuilder::new()
        .with_title("Neovide")
        .with_dimensions(logical_window_size);


    let gl_context = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build_windowed(window_builder, &event_loop)
        .expect("Could not create gl context.");
    let gl_context = unsafe { gl_context.make_current().expect("Could not make gl_context current.") };

    let mut renderer = Renderer::new(
        GLDevice::new(GLVersion::GL3, 0),
        &FilesystemResourceLoader::locate(),
        DestFramebuffer::full_window(window_size),
        RendererOptions { background_color: Some(ColorF::white()) });

    // Make a canvas. We're going to draw a house.
    let mut canvas = CanvasRenderingContext2D::new(CanvasFontContext::from_system_source(),
                                                   window_size.to_f32());

    // Set line width.
    canvas.set_line_width(10.0);

    // Draw walls.
    canvas.stroke_rect(RectF::new(Vector2F::new(75.0, 140.0), Vector2F::new(150.0, 110.0)));

    // Draw door.
    canvas.fill_rect(RectF::new(Vector2F::new(130.0, 190.0), Vector2F::new(40.0, 60.0)));

    // Draw roof.
    let mut path = Path2D::new();
    path.move_to(Vector2F::new(50.0, 140.0));
    path.line_to(Vector2F::new(150.0, 60.0));
    path.line_to(Vector2F::new(250.0, 140.0));
    path.close_path();
    canvas.stroke_path(path);

    // Render the canvas to screen.
    let scene = SceneProxy::from_scene(canvas.into_scene(), RayonExecutor);
    scene.build_and_render(&mut renderer, BuildOptions::default());
    gl_context.swap_buffers().expect("Could not swap buffers");

    // Wait for a keypress.
    event_loop.run_forever(|event| {
        match event {
            _ => ControlFlow::Continue,
        }
    })
}
