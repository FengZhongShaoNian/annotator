use crate::view::{BuildViewFn, ViewId};
use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use crate::dpi::{LogicalPosition, LogicalSize};

pub struct WindowContext {
    /// 按类型存储全局变量
    pub globals_by_type: FxHashMap<TypeId, Box<dyn Any>>,

    /// 提供给BuildViewFn使用
    pub current_view_id: Option<ViewId>,

    /// BuildViewFn中会添加一些命令，这些命令会在BuildViewFn执行完成后被执行
    pub commands: VecDeque<Command>,
}

pub enum Command {
    HideView(ViewId),
    ResizeView(ViewId, LogicalSize<u32>),
    DropView(ViewId),
}

impl Default for WindowContext {
    fn default() -> Self {
        Self {
            globals_by_type: FxHashMap::default(),
            current_view_id: None,
            commands: VecDeque::new(),
        }
    }
}

impl WindowContext {
    pub fn new() -> Self {
        Default::default()
    }
}
