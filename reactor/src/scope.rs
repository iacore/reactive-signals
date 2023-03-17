use arena_link_tree::NodeId;

use crate::Runtime;

#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Copy, Clone)]
pub struct Scope<RT: Runtime> {
    pub(crate) sx: NodeId,
    pub(crate) rt: RT,
}

impl<RT: Runtime> Scope<RT> {
    pub fn new_child(&self) -> Self {
        self.rt.with_mut(|rt| {
            let sx = rt.scope_tree.add_child(self.sx, Default::default());
            Self { sx, rt: rt.id }
        })
    }

    pub fn discard(self) {
        self.rt.with_mut(|rt| {
            let is_root = rt.scope_tree.root() == self.sx;
            if is_root {
                rt.id.discard();
            } else {
                let discarded = rt.scope_tree.discard(self.sx, |s| s.reuse());
                rt.scope_tree
                    .iter_mut_from(rt.scope_tree.root())
                    .for_each(|tree, node| tree[node].remove_scopes(&discarded));
            }
        })
    }
}

impl<RT: Runtime> PartialEq for Scope<RT> {
    #[inline]
    fn eq(&self, other: &Scope<RT>) -> bool {
        self.sx == other.sx
    }
}

impl<RT: Runtime> Eq for Scope<RT> {}
