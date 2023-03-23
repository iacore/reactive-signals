use arena_link_tree::NodeBitVec;

use crate::{runtimes::Runtime, scope::Scope, signal::SignalId, signal::SignalInner, CellType};

#[derive(Debug, Default)]
pub(crate) struct ScopeInner<RT: Runtime> {
    signals: CellType<Vec<SignalInner<RT>>>,
}

impl<RT: Runtime> ScopeInner<RT> {
    /// **Warning!**
    ///
    /// This signal id is not yet valid. There has to be a subsequent
    /// call to `insert_signal` before it is valid
    pub fn next_signal_id(&self, sx: Scope<RT>) -> SignalId<RT> {
        let idx = self.vec_ref().len();
        SignalId::new(idx, sx)
    }

    pub fn insert_signal(&self, signal: SignalInner<RT>) {
        self.vec_mut().push(signal);
    }

    pub fn with_signal<F, T>(&self, id: SignalId<RT>, f: F) -> T
    where
        F: FnOnce(&SignalInner<RT>) -> T,
    {
        let signals = self.vec_ref();
        let signal = signals.get(id.index()).unwrap();
        f(&signal)
    }

    pub(crate) fn remove_scopes(&mut self, discarded_scopes: &NodeBitVec) {
        let signals = self.vec_mut();
        signals
            .iter_mut()
            .for_each(|signal| signal.listeners.retain(|s| !discarded_scopes[s.sx]));
    }

    pub(crate) fn reuse(&self) {
        let signals = self.vec_mut();
        signals.iter_mut().for_each(|signal| signal.reuse());
        signals.clear();
    }
}

#[cfg(not(feature = "unsafe-cell"))]
impl<RT: Runtime> ScopeInner<RT> {
    #[inline]
    fn rt_ref(&self) -> cell::Ref<Vec<SignalInner<RT>>> {
        self.signals.borrow()
    }

    #[inline]
    fn rt_mut(&self) -> cell::RefMut<Vec<SignalInner<RT>>> {
        self.signals.borrow_mut()
    }
}
#[cfg(feature = "unsafe-cell")]
impl<RT: Runtime> ScopeInner<RT> {
    #[inline]
    pub(crate) fn vec_ref(&self) -> &Vec<SignalInner<RT>> {
        unsafe { &*self.signals.get() }
    }

    #[inline]
    fn vec_mut(&self) -> &mut Vec<SignalInner<RT>> {
        unsafe { &mut *self.signals.get() }
    }
}