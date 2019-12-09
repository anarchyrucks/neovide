use std::sync::{Arc, Mutex};
use std::borrow::Cow;

use ggez::{Context, ContextBuilder, GameResult, nalgebra};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Font, FilterMode, Text, Rect, MeshBuilder};

use neovim_lib::NeovimApi;

use crate::editor::{DrawCommand, Editor, Colors};

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

const FONT_NAME: &str = "Delugia Nerd Font";
const FONT_SIZE: f32 = 14.0;
const FONT_WIDTH: f32 = 8.2;
const FONT_HEIGHT: f32 = 16.4;

fn process_draw_commands(ctx: &mut Context, draw_commands: &Vec<DrawCommand>, default_colors: &Colors) {
    let mut mesh_builder = MeshBuilder::new();
    for command in draw_commands {
        let x = command.col_start as f32 * FONT_WIDTH;
        let y = command.row as f32 * FONT_HEIGHT + FONT_HEIGHT;
        let top = y - FONT_HEIGHT * 0.8;
        let width = command.text.chars().count() as f32 * FONT_WIDTH;
        let height = FONT_HEIGHT;
        let background_color = command.style.colors.background.clone().or(default_colors.background.clone()).expect("Could not clone default background");
        mesh_builder.rectangle(DrawMode::fill(), Rect::new(x, top, width, height), background_color);
        let foreground_color = command.style.colors.foreground.clone().or(default_colors.foreground.clone()).unwrap();
        graphics::queue_text(ctx, &Text::new(command.text.clone()), nalgebra::Point2::new(x, y), Some(foreground_color));
    }

    if draw_commands.len() > 0 {
        let mesh = mesh_builder.build(ctx).expect("Could not build background mesh");
        graphics::draw(ctx, &mesh, DrawParam::default());
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear);
    }


    // for command in draw_commands {
    //     let piet_text = piet.text();
    //     let text_layout = piet_text.new_text_layout(&font, &command.text).build().unwrap();
    //     piet.draw_text(&text_layout, (x, y), &command.style.colors.foreground.clone().or(default_colors.foreground.clone()).unwrap());
    // }

}

#[derive(new)]
struct Window {
    editor: Arc<Mutex<Editor>>,
    
}

impl EventHandler for Window {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // graphics::clear(ctx, editor.default_colors.background.clone().unwrap_or(graphics::BLACK));

        let draw_commands = { self.editor.lock().unwrap().build_draw_commands().clone() };
        let default_colors = { self.editor.lock().unwrap().default_colors.clone() };
        process_draw_commands(ctx, &draw_commands, &default_colors);

        graphics::present(ctx)
    }
}
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

    let (mut ctx, mut event_loop) =
        ContextBuilder::new("Neovide", "Keith Simmons")
            .build()
            .unwrap();

    let resources: Vec<Cow<'static, str>> = Resources::iter().collect();

    let font_binary = Resources::get("Delugia.Nerd.Font.Complete.ttf").expect("Could not load resource");
    graphics::Font::new_glyph_font_bytes(&mut ctx, font_binary.as_ref()).expect("Could not load font");

    graphics::set_resizable(&mut ctx, true);
    let mut window = Window::new(editor);

    match event::run(&mut ctx, &mut event_loop, &mut window) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occurred: {}", e)
    };
}
