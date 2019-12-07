use std::sync::{Arc, Mutex};
use neovim_lib::NeovimApi;

use render::*;
use widget::*;

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

struct App {
    editor: Arc<Mutex<Editor>>,

    time: f32,
    animation: Anim,

    background: Quad,
    desktop_window: DesktopWindow,
    animator: Animator
}

impl App {
    pub fn proto(cx: &mut Cx, editor: Arc<Mutex<Editor>>) -> Self {
        set_widget_style(cx, &StyleOptions::default());
        Self {
            editor,

            time: 0.0,
            animation: Anim::new(Play::Forever { duration: std::f64::INFINITY, cut: false,  term: false }, Vec::new()),

            background: Quad::proto(cx),
            desktop_window: DesktopWindow {
                caption: "Neovide".to_string(),
                window: Window::proto(cx),
                ..DesktopWindow::proto(cx)
            },
            animator: Animator::default()
        }
    }

    fn handle_app(&mut self, cx: &mut Cx, event: &mut Event) {
        self.desktop_window.handle_desktop_window(cx, event);
        self.animator.play_anim(cx, self.animation.clone());
    }

    fn draw_app(&mut self, cx: &mut Cx) {
        if (self.desktop_window.begin_desktop_window(cx, None)).is_err() {
            return;
        };

        let size = self.desktop_window.window.get_inner_size(cx);

        self.animator.init(cx, |_| Anim::new(Play::Forever { duration: std::f64::INFINITY, cut: false,  term: false }, Vec::new()));

        self.background.color = color("white");
        let area = self.background.draw_quad_rel(cx, Rect { 
            x: 0.0, 
            y: 0.0, 
            w: size.x, 
            h: size.y
        });
        self.background.color = color("blue");
        self.background.draw_quad_rel(cx, Rect {
            x: 50.0 + self.time.cos() * 50.0,
            y: 50.0 + self.time.sin() * 50.0,
            w: 100.0,
            h :100.0
        });

        self.animator.set_area(cx, area.into());
        self.desktop_window.end_desktop_window(cx);
        self.time = self.time + 0.01;
    }
}

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
    let mut cx = Cx::default();
    let mut app = App::proto(&mut cx, editor);
    let mut cxafterdraw = CxAfterDraw::proto(&mut cx);
    cx.event_loop( | cx, mut event | {
        if let Event::Draw = event {
            app.draw_app(cx);
            cxafterdraw.after_draw(cx);
            return
        }
        app.handle_app(cx, &mut event);
    });
}
