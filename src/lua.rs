use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic, Table};

use imgui::*;

use crate::imgui_wrapper::*;

pub struct MpLua {
    lua: Lua,
}

impl MpLua {
    pub fn new(file_content: &str) -> Self {
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            lua_ctx.load(&file_content).exec().expect("loading lua files");
        });
        MpLua { lua }
    }

    pub fn make_render<'ui>(&self, ui: &'ui imgui::Ui) -> Box<dyn FnOnce() -> () + 'ui> {
        let mut ui_str = String::from("");
        self.lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let func_update = globals.get::<_, Function>("update").expect("gettting lua entry");
            func_update.call::<_, ()>(0).expect("calling update");
            ui_str = globals.get::<_, Table>("mp_state").expect("getting mp_state").get("num").unwrap();
        });
        Box::new(move || {
            ui.text(ui_str);
        })
    }
}
