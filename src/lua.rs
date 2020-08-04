use path_slash::PathBufExt;
use rlua::{Function, Integer, Lua, Table, Value};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use imgui::im_str;

#[derive(Debug)]
enum UiStatusItem {
    Text(String),
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
                    self.items.push(UiStatusItem::Text(format!(
                        "{}{}: ",
                        indent,
                        display_value(&key)
                    )));
                    self.build(inner_table, depth + 1)?;
                }
                _ => {
                    self.items.push(UiStatusItem::Text(format!(
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
                text: text,
            })
        }
        Ok(())
    }
}

pub struct MpLua {
    lua: Lua,
    entry_file: PathBuf,
}

impl MpLua {
    pub fn new(entry_file: String) -> Self {
        let lua = Lua::new();
        let path = PathBuf::from(entry_file);
        let mut mp_lua = MpLua {
            lua: lua,
            entry_file: path,
        };
        mp_lua.add_require_path().unwrap();
        mp_lua.load().unwrap();
        mp_lua
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
        println!("{}", lua);

        self.lua.context(|lua_ctx| lua_ctx.load(&lua).exec())?;
        Ok(())
    }

    fn build_ui_status(&self) -> rlua::Result<UiStatus> {
        let mut info = UiStatus { items: vec![] };
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let func_update = globals.get::<_, Function>("update")?;
            func_update.call::<_, ()>(0)?;
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

    pub fn make_status_render<'ui>(&self, ui: &'ui imgui::Ui) -> Box<dyn FnOnce() -> () + 'ui> {
        match self.build_ui_status() {
            Ok(status) => {
                Box::new(move || {
                    // ui.text(ui_str);
                    for item in status.items {
                        match item {
                            UiStatusItem::Text(text) => {
                                ui.text(text);
                            }
                        }
                    }
                })
            }
            Err(e) => {
                println!("{:?}", e);
                Box::new(move || {})
            }
        }
    }

    pub fn run_selection(&self, index: usize) -> rlua::Result<()> {
        &self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_selection = globals.get::<_, Table>("mp_selection")?;
            let func = mp_selection
                .get::<_, Table>(index)?
                .get::<_, Function>("callback")?;
            func.call::<(), ()>(())
        })?;
        Ok(())
    }

    pub fn make_slection_render<'ui>(
        &'ui self,
        ui: &'ui imgui::Ui,
    ) -> Box<dyn FnOnce() -> () + 'ui> {
        match self.build_ui_selection() {
            Ok(selection) => Box::new(move || {
                for item in selection.items {
                    match item {
                        UiSelectionItem::Button { index, text } => {
                            if ui.button(&im_str!("{}", &text), [100f32, 30f32]) {
                                log_lua_result(&self.run_selection(index));
                            }
                        }
                    }
                }
            }),
            Err(e) => {
                println!("{:?}", e);
                Box::new(move || {})
            }
        }
    }
}
