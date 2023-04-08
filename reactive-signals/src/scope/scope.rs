use std::ptr::NonNull;

use crate::arena_tree::NodeId;
use crate::Runtime;

///
/// [Signal](crate::Signal)s are created in scopes and can only be deleted by
/// discarding the scope.
///
/// Scopes are created in a tree structure, where the root scope is created by one of the [runtimes](crate::runtimes),
/// and child scopes can be added to any Scope by calling the [new_child()](Self::new_child()) function on a scope.
///
/// When calling a Scope's [discard()](Self::discard()) function, the Scope and it's child scopes are discarded
/// together with their signals.
///
/// Internally, a Scope is really just a u16 index into an arena based tree which contains the
/// full ScopeInner data (not exposed in the api doc). The Scope implements [Copy] which makes it
/// much easier to use in closures.
///
/// There can be a maximum of 65k Scopes.
///
/// ## Typed attached data
///
/// > _**This has not been implemented!** A proof of concept has been done but the exact
/// > details of how the api will work are not cemented yet._
///
///
/// It is possible to attach data to a Scope and then, in a type-safe and performant manner, access it.
/// When attached to a Scope the data gets transformed into a Signal which can be retrieved
/// with a function named as the data struct but snake-cased.
///
/// You can add several nested data values to a scope. The cost of adding one is 2 bytes added
/// to the scope id which is the vector index of the signal plus the cost of the Signal added to
/// the ScopeInner.
///
/// ```ignore
/// // derive using Scoped with the input scope type as argument
/// // the generated scope wrapper is named MyCounterScope
/// #[derive(Scoped(Scope), Clone, Copy)]
/// struct MyCounter(u8);
///
/// // a derive with the name argument as well
/// #[derive(Scoped(MyCounterScope, name = "BaseScope"), Clone)]
/// struct MyGreeting(String);
///
/// fn some_func<RT: Runtime>(sc: Scope<RT>) {
///     // create your data
///     let count = MyCounter(0);
///
///     // attach it to the scope (type annotations not necessary)
///     let sc: MyCounterScope<RT: Runtime> = count.attach_to(sc);
///     
///     // The MyCount instance can be accessed as a signal (type annotations not necessary)
///     let count_signal: Signal<EqData<i32>, RT> = sc.my_counter();
///     
///     // Create a MyGreeting and attach it to the MyCounterScope
///     let sc: BaseScope<RT: Runtime> = MyGreeting("hi ".to_string()).attach_to(sc);
///
///     next_func(sc);
/// }
///
/// // The scope is passed as a typed parameter
/// fn next_func<RT: Runtime>(sc: BaseScope<RT>) {
///     // the scoped data can be modified
///     sc.my_greeting().update(|s| *s = *s.trim());
///
///     signal!(sc, move || {
///         sc.my_greeting().with(|greet| println!("{greet} {} times", sc.my_counter().get()))
///     });
/// }
/// ```
///

pub struct RootScopeGuard(&'static Runtime);

impl Drop for RootScopeGuard {
    fn drop(&mut self) {
        self.0.with_mut(|inner| inner.discard());
        // the official rust docs proposes to use this to
        // drop something previously leaked
        // https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak
        // but on the rust discord there's many different opinions.
        let nn = NonNull::from(self.0);
        let b = unsafe { Box::from_raw(nn.as_ptr()) };
        drop(b);
    }
}

#[derive(Copy, Clone)]
pub struct Scope {
    pub(crate) sx: NodeId,
    pub(crate) rt: &'static Runtime,
}

impl Scope {
    pub fn new_child(&self) -> Self {
        let sx = self
            .rt
            .with_mut(|rt| rt.scope_tree.add_child(self.sx, Default::default()));

        Self { sx, rt: self.rt }
    }

    pub fn discard(self) {
        self.rt.with_mut(|rt| {
            let is_root = rt.scope_tree.root() == self.sx;
            if is_root {
                rt.discard();
            } else {
                let discarded = rt.scope_tree.discard(self.sx, |s| s.reuse());
                let id = rt.scope_tree.root();
                rt.scope_tree
                    .iter_mut_from(id)
                    .for_each(|tree, node| tree[node].remove_scopes(&discarded));
            }
        });
    }

    pub fn new_client_side_root_scope() -> (RootScopeGuard, Scope) {
        Self::new(true)
    }

    pub fn new_server_side_root_scope() -> (RootScopeGuard, Scope) {
        Self::new(false)
    }

    fn new(client_side: bool) -> (RootScopeGuard, Scope) {
        let rt = Runtime::new(client_side);
        let guard = RootScopeGuard(rt);
        let scope = rt.new_root_scope();
        (guard, scope)
    }
}
