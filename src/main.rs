mod editor;
mod window;

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate rust_embed;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use env_logger::Env as LoggerEnv;
use neovim_lib::{Neovim, UiAttachOptions, Session};
use rmpv::Value;

use window::ui_loop;
use editor::{Colors, Editor, GridLineCell, Style};

use ggez::graphics::Color;

const INITIAL_WIDTH: u16 = 100;
const INITIAL_HEIGHT: u16 = 50;

fn handle_grid_line(grid_line_arguments: &Vec<Value>, editor: &Arc<Mutex<Editor>>) {
    if let [Value::Integer(grid_id), Value::Integer(row), Value::Integer(col_start), Value::Array(cells)] = grid_line_arguments.as_slice() {
        let mut col_pos = col_start.as_u64().unwrap() as u16;
        for cell in cells.into_iter() {
            if let Value::Array(cell_data) = cell {
                let grid_id = grid_id.as_u64().unwrap() as u16;
                let row = row.as_u64().unwrap() as u16;

                let mut text = match cell_data.get(0).expect("Cell must have non zero size") {
                    Value::String(cell_text) => cell_text.as_str().expect("Could not process string").to_string(),
                    _ => panic!("Cell text was not a string")
                };

                if let Some(Value::Integer(repeat_count)) = cell_data.get(2) {
                    text = text.repeat(repeat_count.as_u64().unwrap_or(1) as usize);
                }

                let mut style_id = None;
                if let Some(Value::Integer(id)) = cell_data.get(1) {
                    style_id = Some(id.as_u64().unwrap());
                }

                let mut editor = editor.lock().unwrap();
                let length = text.chars().count() as u16;
                editor.draw(GridLineCell::new(grid_id, text, row, col_pos, style_id));
                col_pos = col_pos + length;
            } else {
                println!("Invalid grid_line cell format: {:?}", cell);
            }
        }
    } else {
        println!("Invalid grid_line format: {:?}", grid_line_arguments);
    }
}

fn handle_clear(_clear_arguments: &Vec<Value>, editor: &Arc<Mutex<Editor>>) {
    let mut editor = editor.lock().unwrap();
    editor.clear();
}

fn handle_cursor_goto(cursor_goto_arguments: &Vec<Value>, editor: &Arc<Mutex<Editor>>) {
    if let [Value::Integer(_grid_id), Value::Integer(row), Value::Integer(column)] = cursor_goto_arguments.as_slice() {
        let mut editor = editor.lock().unwrap();
        editor.jump_cursor_to(column.as_u64().unwrap() as u16, row.as_u64().unwrap() as u16);
    } else {
        println!("Invalid cursor_goto format: {:?}", cursor_goto_arguments);
    }
}

fn unpack_color(packed_color: u64) -> Color {
    Color::from_rgba_u32(((packed_color as u32) << 8) + 255)
}

fn handle_default_colors(default_colors_arguments: &Vec<Value>, editor: &Arc<Mutex<Editor>>) {
    if let [
        Value::Integer(foreground), Value::Integer(background), Value::Integer(special), 
        Value::Integer(_term_foreground), Value::Integer(_term_background)
    ] = default_colors_arguments.as_slice() {
        let foreground = unpack_color(foreground.as_u64().unwrap());
        let background = unpack_color(background.as_u64().unwrap());
        let special = unpack_color(special.as_u64().unwrap());

        let mut editor = editor.lock().unwrap();
        editor.set_default_colors(foreground, background, special);
    } else {
        println!("Invalid default color format.");
    }
}

fn handle_hl_attr_define(hl_attr_define_arguments: &Vec<Value>, editor: &Arc<Mutex<Editor>>) {
    if let [
        Value::Integer(id), Value::Map(attributes), Value::Map(_terminal_attributes), Value::Array(_info)
    ] = hl_attr_define_arguments.as_slice() {
        let id = id.as_u64().unwrap();
        let mut editor = editor.lock().unwrap();
        let mut style = Style::new(Colors::new(None, None, None));
        for attribute in attributes {
            if let (Value::String(name), value) = attribute {
                match (name.as_str().unwrap(), value) {
                    ("foreground", Value::Integer(packed_color)) => style.colors.foreground = Some(unpack_color(packed_color.as_u64().unwrap())),
                    ("background", Value::Integer(packed_color)) => style.colors.background = Some(unpack_color(packed_color.as_u64().unwrap())),
                    ("special", Value::Integer(packed_color)) => style.colors.special = Some(unpack_color(packed_color.as_u64().unwrap())),
                    _ => println!("Ignored style attribute: {}", name)
                }
            } else {
                println!("Invalid attribute format");
            }
        }
        editor.define_style(id, style);
    }
}

fn handle_redraw_event(event_value: Value, editor: &Arc<Mutex<Editor>>) {
    match event_value {
        Value::Array(event_contents) => {
            let name_value = &event_contents[0];
            let events = &event_contents[1..];
            for event in events {
                match (name_value, event) {
                    (Value::String(event_name), Value::Array(arguments)) => {
                        match event_name.as_str().expect("Invalid redraw command name format.") {
                            "grid_resize" => println!("grid_resize event ignored"),
                            "default_colors_set" => handle_default_colors(arguments, editor),
                            "hl_attr_define" => handle_hl_attr_define(arguments, editor),
                            "grid_line" => handle_grid_line(arguments, &editor),
                            "grid_clear" => handle_clear(arguments, &editor),
                            "grid_cursor_goto" => handle_cursor_goto(arguments, &editor),
                            "hl_group_set" => println!("hl_group_set event ignored"),
                            other => println!("Unhandled redraw command {}", other)
                        }
                    },
                    _ => {
                        println!("Unrecognized redraw event structure.");
                    }
                }
            }
        },
        _ => println!("Event is not an array...")
    }
}

fn nvim_event_loop(receiver: Receiver<(String, Vec<Value>)>, editor: &Arc<Mutex<Editor>>) {
    println!("UI thread spawned");
    loop {
        let (event_name, event_args) = receiver.recv().expect("Could not receive event.");
        match event_name.as_ref() {
            "redraw" => {
                for event in event_args {
                    handle_redraw_event(event, &editor);
                }
            },
            _ => println!("Unrecognized Event: {}", event_name)
        };
    }
}

fn main() {
    env_logger::from_env(LoggerEnv::default().default_filter_or("warn")).init();

    let mut session = Session::new_child().unwrap();
    let receiver = session.start_event_loop_channel();
    let mut nvim = Neovim::new(session);
    let mut options = UiAttachOptions::new();
    options.set_cmdline_external(false);
    options.set_messages_external(false);
    options.set_linegrid_external(true);
    options.set_rgb(true);
    nvim.ui_attach(INITIAL_WIDTH as i64, INITIAL_HEIGHT as i64, &options).unwrap();

    let editor = Arc::new(Mutex::new(Editor::new(nvim, INITIAL_WIDTH, INITIAL_HEIGHT)));

    let nvim_editor = editor.clone();
    thread::spawn(move || {
        nvim_event_loop(receiver, &nvim_editor);
    });

    ui_loop(editor);
}
