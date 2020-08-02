use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic};

use imgui::*;

use crate::imgui_wrapper::*;

pub struct MpLua {
    lua: Lua,
}

impl MpLua {
    pub fn new(file_content: &str) -> Self {
        MpLua {
            lua: Lua::new()
        }
    }

    pub fn make_render<'ui>(&self, ui: &'ui imgui::Ui) -> Box<dyn FnOnce() -> () + 'ui> {
        Box::new(move || {
            ui.text(im_str!("i love lua"));
        })
    }
}