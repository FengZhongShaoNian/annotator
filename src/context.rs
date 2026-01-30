use crate::global::Global;
use anyhow::Context;
use rustc_hash::FxHashMap;
use std::any::{type_name, Any, TypeId};

#[derive(Default)]
pub struct WindowContext {
    /// 按类型存储全局变量
    globals_by_type: FxHashMap<TypeId, Box<dyn Any>>,
}

impl WindowContext {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn global<G: Global>(&self) -> &G{
        self.globals_by_type
            .get(&TypeId::of::<G>())
            .map(|any_state| any_state.downcast_ref::<G>().unwrap())
            .with_context(|| format!("no state of type {} exists", type_name::<G>()))
            .unwrap()
    }

    pub fn global_mut<G: Global>(&mut self) -> &mut G{
        self.globals_by_type
        .get_mut(&TypeId::of::<G>())
        .map(|any_state| any_state.downcast_mut::<G>().unwrap())
        .with_context(|| format!("no state of type {} exists", type_name::<G>()))
        .unwrap()
    }

    pub fn set_global<G: Global>(&mut self, global: G) {
        self.globals_by_type.insert(TypeId::of::<G>(), Box::new(global));
    }

    pub fn has_global<G: Global>(&self) -> bool {
        self.globals_by_type.contains_key(&TypeId::of::<G>())
    }

    pub fn get_global_or_insert_with<F,G: Global>(&mut self, func: F) -> &G
    where F: FnOnce() -> G {
        if self.has_global::<G>() {
            self.global()
        } else {
            let global = func();
            self.set_global(global);
            self.global_mut()
        }
    }
}
