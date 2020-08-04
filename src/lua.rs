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
    pub fn build(&mut self, lua_table: Table, depth: u32) {
        for pair in lua_table.pairs::<Value, Value>() {
            let (key, value) = pair.expect("getting pair");
            let indent = " ".repeat((depth * 2) as usize);
            match value {
                Value::Table(inner_table) => {
                    self.items.push(UiStatusItem::Text(format!(
                        "{}{}: ",
                        indent,
                        display_value(&key)
                    )));
                    self.build(inner_table, depth + 1);
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
            // ...
        }
    }
}

enum UiSelectionItem {
    Button { index: usize, text: String },
}

struct UiSelection {
    items: Vec<UiSelectionItem>,
}

impl UiSelection {
    pub fn build(&mut self, lua_table: Table) {
        for pair in lua_table.pairs::<Integer, Table>() {
            let (index, selection_table) = pair.expect("getting pair");
            let text = selection_table
                .get::<_, String>("text")
                .expect("getting selection text");
            self.items.push(UiSelectionItem::Button {
                index: index as usize,
                text: text,
            })
        }
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
        self.lua.context(|lua_ctx| {
            lua_ctx
                .load(&file_content)
                .exec()
                .expect("loading lua files")
        });
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

        self.lua.context(|lua_ctx| {
            lua_ctx.load(&lua).exec().expect("add require path to lua");
        });
        Ok(())
    }

    fn build_ui_status(&self) -> UiStatus {
        let mut info = UiStatus { items: vec![] };
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let func_update = globals
                .get::<_, Function>("update")
                .expect("gettting lua entry");
            func_update.call::<_, ()>(0).expect("calling update");
            let state = globals
                .get::<_, Table>("mp_state")
                .expect("getting mp_state");
            info.build(state, 0u32);
        });

        info
    }

    fn build_ui_selection(&self) -> UiSelection {
        let mut selection = UiSelection { items: vec![] };
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_selection = globals
                .get::<_, Table>("mp_selection")
                .expect("getting mp_selection");
            selection.build(mp_selection);
            // info.build(state, 0u32);
        });

        selection
    }

    pub fn make_status_render<'ui>(&self, ui: &'ui imgui::Ui) -> Box<dyn FnOnce() -> () + 'ui> {
        let info = self.build_ui_status();
        Box::new(move || {
            // ui.text(ui_str);
            for item in info.items {
                match item {
                    UiStatusItem::Text(text) => {
                        ui.text(text);
                    }
                }
            }
        })
    }

    pub fn run_selection(&self, index: usize) {
        &self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let mp_selection = globals
                .get::<_, Table>("mp_selection")
                .expect("renderer get mp_selection");
            let func = mp_selection
                .get::<_, Table>(index)
                .expect("renderer get table")
                .get::<_, Function>("callback")
                .expect("renderer get callback");
            func.call::<(), ()>(()).expect("renderer call callback");
        });
    }

    pub fn make_slection_render<'ui>(
        &'ui self,
        ui: &'ui imgui::Ui,
    ) -> Box<dyn FnOnce() -> () + 'ui> {
        let selection = self.build_ui_selection();
        Box::new(move || {
            for item in selection.items {
                match item {
                    UiSelectionItem::Button { index, text } => {
                        if ui.button(&im_str!("{}", &text), [100f32, 30f32]) {
                            self.run_selection(index);
                        }
                    }
                }
            }
        })
    }
}
