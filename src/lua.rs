use rlua::{
    prelude::FromLua, Context, Function, Lua, MetaMethod, Result, Table, UserData, UserDataMethods,
    Value, Variadic,
};

use imgui::*;

use crate::imgui_wrapper::*;

#[derive(Debug)]
enum InfoItem {
    Text(String),
}

struct Info {
    items: Vec<InfoItem>,
}

fn display_value<'lua>(value: &'lua Value) -> String {
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

impl Info {
    pub fn build(&mut self, lua_table: Table, depth: u32) {
        for pair in lua_table.pairs::<Value, Value>() {
            let (key, value) = pair.expect("getting pair");
            match value {
                Value::Table(inner_table) => {
                    self.items
                        .push(InfoItem::Text(format!("{}: ", display_value(&key))));
                    self.build(inner_table, depth + 1);
                }
                _ => {
                    self.items.push(InfoItem::Text(format!(
                        "{}{}: {}",
                        " ".repeat((depth * 2) as usize),
                        display_value(&key),
                        display_value(&value)
                    )));
                }
            }
            // ...
        }
    }
}

pub struct MpLua {
    lua: Lua,
}

impl MpLua {
    pub fn new(file_content: &str) -> Self {
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            lua_ctx
                .load(&file_content)
                .exec()
                .expect("loading lua files");
        });
        MpLua { lua }
    }

    fn build_info(&self) -> Info {
        let mut info = Info { items: vec![] };
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
            // = .get("num").unwrap();
        });

        info
    }

    pub fn make_render<'ui>(&self, ui: &'ui imgui::Ui) -> Box<dyn FnOnce() -> () + 'ui> {
        let mut ui_str = String::from("");
        let info = self.build_info();
        Box::new(move || {
            // ui.text(ui_str);
            for item in info.items {
                match item {
                    InfoItem::Text(text) => {
                        ui.text(text);
                    }
                }
            }
        })
    }
}
