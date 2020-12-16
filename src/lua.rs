use path_slash::PathBufExt;
use rlua::{Function, Integer, Lua, Table, Value};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use std::rc::Rc;

use ggez::Context;
use imgui::{im_str, ImString};

use crate::signal::{SIGNAL_RELOAD_SELECTION, SIGNAL_TABLE};

#[derive(Debug)]
enum UiStatusItem {
    Text(ImString),
}

pub fn log_lua_result(result: &rlua::Result<()>) {
    if let Err(e) = result {
        println!("[LuaError]{}", e);
    }
}

struct UiStatus {
    items: Vec<UiStatusItem>,
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Table(_) => String::from("table"),
        Value::String(str) => String::from(str.to_str().unwrap_or("")),
        Value::Boolean(b) => format!("{}", b),
        Value::Nil => String::from("Nil"),
        Value::Number(n) => format!("{}", n),
        Value::Integer(n) => format!("{}", n),
        _ => format!("{:?}", value),
    }
}

impl UiStatus {
    pub fn build(&mut self, lua_table: Table, depth: u32) -> rlua::Result<()> {
        for pair in lua_table.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let indent = " ".repeat((depth * 2) as usize);
            match value {
                Value::Table(inner_table) => {
                    self.items.push(UiStatusItem::Text(im_str!(
                        "{}{}: ",
                        indent,
                        display_value(&key)
                    )));
                    self.build(inner_table, depth + 1)?;
                }
                _ => {
                    // println!("value = {}", display_value(&value));
                    self.items.push(UiStatusItem::Text(im_str!(
                        "{}{}: {}",
                        indent,
                        display_value(&key),
                        display_value(&value)
                    )));
                }
            }
        }
        Ok(())
    }
}

enum UiSelectionItem {
    Button { index: usize, text: String },
}

struct UiSelection {
    items: Vec<UiSelectionItem>,
}

impl UiSelection {
    pub fn build(&mut self, lua_table: Table) -> rlua::Result<()> {
        for pair in lua_table.pairs::<Integer, Table>() {
            let (index, selection_table) = pair?;
            let text = selection_table.get::<_, String>("text")?;
            self.items.push(UiSelectionItem::Button {
                index: index as usize,
                text,
            })
        }
        Ok(())
    }
}

const LED_SIZE: usize = 16;

pub struct Led {
    pub buf: [bool; LED_SIZE * LED_SIZE],
}

impl Default for Led {
    fn default() -> Self {
        Led {
            buf: [false; LED_SIZE * LED_SIZE],
        }
    }
}

impl Led {
    pub fn build(&mut self, lua_table: Table) -> rlua::Result<()> {
        for pair in lua_table.pairs::<Integer, bool>() {
            let (index, is_on) = pair?;
            if index >= 0 && (index as usize) < self.buf.len() {
                self.buf[index as usize] = is_on;
            }
        }
        Ok(())
    }
}

pub struct MpLua {
    lua: Lua,
    entry_file: PathBuf,
    selections: Option<Rc<UiSelection>>,
}

impl MpLua {
    pub fn new(entry_file: String) -> Self {
        let lua = Lua::new();
        let path = PathBuf::from(entry_file);
        let mut mp_lua = MpLua {
            lua,
            entry_file: path,
            selections: None,
        };
        mp_lua.add_require_path().unwrap();
        if let Err(e) = mp_lua.load() {
            panic!("{}", e);
        }
        mp_lua
    }

    pub fn awake(&mut self) -> rlua::Result<()> {
        self.load_ui_selection()?;
        self.clear_signals()?;
        self.inject_functions()?;
        self.lua.load_from_std_lib(rlua::StdLib::STRING)?;
        self.run_awake()?;
        Ok(())
    }

    fn inject_functions(&mut self) -> rlua::Result<()> {
        let mp_lib = std::include_bytes!("../resources/lua/signal.lua");
        self.lua.context(|lua_ctx| {
            lua_ctx
                .load(&String::from_utf8_lossy(mp_lib).into_owned())
                .exec()?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn tick_signal(&mut self) -> rlua::Result<()> {
        let mut signals = vec![];
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let lua_signals = globals.get::<_, Table>(SIGNAL_TABLE)?;
            for pair in lua_signals.pairs::<Value, Value>() {
                let (_, signal) = pair?;
                if let Value::Integer(s) = signal {
                    signals.push(s);
                }
            }
            Ok(())
        })?;
        for signal in signals {
            self.run_signal(signal)?;
        }
        self.clear_signals()?;
        Ok(())
    }

    fn clear_signals(&mut self) -> rlua::Result<()> {
        self.lua.context(|lua_ctx| {
            lua_ctx.load(&format!("{} = {{}}", SIGNAL_TABLE)).exec()?;
            Ok(())
        })?;
        Ok(())
    }

    fn run_signal(&mut self, signal: i64) -> rlua::Result<()> {
        match signal {
            SIGNAL_RELOAD_SELECTION => {
                self.load_ui_selection()?;
            }
            _ => {}
        };
        Ok(())
    }

    fn load_ui_selection(&mut self) -> rlua::Result<()> {
        let selections = self.build_ui_selection()?;
        self.selections.replace(Rc::new(selections));
        Ok(())
    }

    fn run_awake(&mut self) -> rlua::Result<()> {
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let awake_func = globals.get::<_, Function>("awake")?;
            awake_func.call::<_, ()>(())?;
            Ok(())
        })?;
        Ok(())
    }

    fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let file_content = fs::read_to_string(&self.entry_file)?;
        self.lua
            .context(|lua_ctx| lua_ctx.load(&file_content).exec())?;
        Ok(())
    }

    fn add_require_path(&mut self) -> Result<(), Box<dyn Error>> {
        let mut path = self.entry_file.clone();
        path.pop();
        let lua_path_base = path.to_slash().unwrap();
        let lua = format!(
            r#"package.path = package.path..';'..'./{}/?.lua' print(package.path)"#,
            lua_path_base
        );
        self.lua.context(|lua_ctx| lua_ctx.load(&lua).exec())?;
        Ok(())
    }

    fn build_ui_status(&self, ctx: &Context) -> rlua::Result<UiStatus> {
        let mut info = UiStatus { items: vec![] };
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let func_update = globals.get::<_, Function>("update")?;
            let delta = ggez::timer::delta(&ctx).as_secs_f64();
            let time_since_start = ggez::timer::time_since_start(&ctx).as_secs_f64();
            func_update.call::<_, ()>((delta, time_since_start))?;
            let state = globals.get::<_, Table>("mp_state")?;
            info.build(state, 0u32)?;
            Ok(())
        })?;

        Ok(info)
    }

    fn build_ui_selection(&self) -> rlua::Result<UiSelection> {
        let mut selection = UiSelection { items: vec![] };
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_selection = globals.get::<_, Table>("mp_selection")?;
            selection.build(mp_selection)?;
            Ok(())
        })?;
        Ok(selection)
    }

    fn build_ui_led(&self) -> rlua::Result<Led> {
        let mut led: Led = Default::default();
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_led = globals.get::<_, Table>("mp_led")?;
            led.build(mp_led)?;
            Ok(())
        })?;
        Ok(led)
    }

    pub fn make_status_render<'ui>(
        &self,
        ui: &'ui imgui::Ui,
        ctx: &Context,
    ) -> Box<dyn FnOnce() -> () + 'ui> {
        match self.build_ui_status(ctx) {
            Ok(status) => Box::new(move || {
                for item in status.items {
                    match item {
                        UiStatusItem::Text(text) => {
                            ui.text(text);
                        }
                    }
                }
            }),
            Err(e) => {
                println!("make render: {:?}", e);
                Box::new(move || {})
            }
        }
    }

    pub fn run_selection(&self, index: usize) -> rlua::Result<()> {
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_selection = globals.get::<_, Table>("mp_selection")?;
            let func = mp_selection
                .get::<_, Table>(index)?
                .get::<_, Function>("callback")?;
            func.call::<(), ()>(())
        })
    }

    pub fn make_slection_render<'ui>(
        &'ui self,
        ui: &'ui imgui::Ui,
    ) -> Box<dyn FnOnce() -> () + 'ui> {
        match &self.selections {
            Some(rc_selections) => Box::new(move || {
                for item in &rc_selections.items {
                    match item {
                        UiSelectionItem::Button { index, text } => {
                            if ui.button(&im_str!("{}", &text), [200f32, 30f32]) {
                                log_lua_result(&self.run_selection(*index));
                            }
                        }
                    }
                }
            }),
            None => Box::new(move || {}),
        }
    }

    pub fn make_led_render<'ui>(
        &'ui self,
        ui: &'ui imgui::Ui,
        cell_size: f32,
    ) -> Box<dyn FnOnce() -> () + 'ui> {
        match self.build_ui_led() {
            Ok(led) => Box::new(move || {
                let c0 = [0.5, 1.0, 1.0, 1.0];
                let c1 = [0.1, 0.1, 0.1, 1.0];
                let draw_list = ui.get_window_draw_list();
                let win_pos = ui.window_pos();
                for (i, b) in led.buf.iter().enumerate() {
                    let row = (i / LED_SIZE) as f32;
                    let col = (i % LED_SIZE) as f32;
                    let x1 = row * cell_size + win_pos[0] + 32.0;
                    let y1 = col * cell_size + win_pos[1] + 32.0;
                    draw_list.add_rect_filled_multicolor(
                        [x1, y1],
                        [x1 + cell_size - 4.0, y1 + cell_size - 4.0],
                        if *b { c0 } else { c1 },
                        if *b { c0 } else { c1 },
                        if *b { c0 } else { c1 },
                        if *b { c0 } else { c1 },
                    );
                }
            }),
            Err(e) => {
                println!("make led render: {:?}", e);
                Box::new(move || {})
            }
        }
    }
}
