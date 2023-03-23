use crate::{scope::Scope, CellType};

use super::{Runtime, RuntimeInner};

pub struct StaticRuntime(StaticRuntimeId);

#[derive(Clone, Copy)]
pub struct StaticRuntimeId(&'static CellType<RuntimeInner<StaticRuntimeId>>);

impl Default for StaticRuntimeId {
    fn default() -> Self {
        todo!()
    }
}

impl Runtime for StaticRuntimeId {
    #[inline]
    fn with_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut RuntimeInner<StaticRuntimeId>) -> T,
    {
        f(&mut self.rt_mut())
    }

    #[inline]
    fn with_ref<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&RuntimeInner<StaticRuntimeId>) -> T,
    {
        f(&self.rt_ref())
    }
}

impl StaticRuntime {
    pub fn new_root_scope() -> Scope<StaticRuntimeId> {
        let mut rti = RuntimeInner::new();
        let sx = rti.scope_tree.init(Default::default());
        let rt = StaticRuntimeId(Box::leak(Box::new(CellType::new(rti))));
        Scope { sx, rt }
    }
}

#[cfg(not(feature = "unsafe-cell"))]
impl StaticRuntimeId {
    #[inline]
    fn rt_ref(&self) -> cell::Ref<RuntimeInner<StaticRuntimeId>> {
        self.0.borrow()
    }

    #[inline]
    fn rt_mut(&self) -> cell::RefMut<RuntimeInner<StaticRuntimeId>> {
        self.0.borrow_mut()
    }
}
#[cfg(feature = "unsafe-cell")]
impl StaticRuntimeId {
    #[inline]
    fn rt_ref(&self) -> &RuntimeInner<StaticRuntimeId> {
        unsafe { &*self.0.get() }
    }

    #[inline]
    fn rt_mut(&self) -> &mut RuntimeInner<StaticRuntimeId> {
        unsafe { &mut *self.0.get() }
    }
}