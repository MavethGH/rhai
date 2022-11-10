//! Global runtime state.

use crate::{Dynamic, Engine, ImmutableString, SharedModule, StaticVec};
use std::fmt;
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

/// Collection of globally-defined constants.
#[cfg(not(feature = "no_module"))]
#[cfg(not(feature = "no_function"))]
pub type GlobalConstants =
    crate::Shared<crate::Locked<std::collections::BTreeMap<ImmutableString, Dynamic>>>;

/// _(internals)_ Global runtime states.
/// Exported under the `internals` feature only.
//
// # Implementation Notes
//
// This implementation for imported [modules][crate::Module] splits the module names from the shared
// modules to improve data locality.
//
// Most usage will be looking up a particular key from the list and then getting the module that
// corresponds to that key.
#[derive(Clone)]
pub struct GlobalRuntimeState {
    /// Names of imported [modules][crate::Module].
    #[cfg(not(feature = "no_module"))]
    imports: StaticVec<ImmutableString>,
    /// Stack of imported [modules][crate::Module].
    #[cfg(not(feature = "no_module"))]
    modules: StaticVec<SharedModule>,
    /// The current stack of loaded [modules][Module].
    pub lib: StaticVec<SharedModule>,
    /// Source of the current context.
    ///
    /// No source if the string is empty.
    pub source: Option<ImmutableString>,
    /// Number of operations performed.
    pub num_operations: u64,
    /// Number of modules loaded.
    #[cfg(not(feature = "no_module"))]
    pub num_modules_loaded: usize,
    /// The current nesting level of function calls.
    pub level: usize,
    /// Level of the current scope.
    ///
    /// The global (root) level is zero, a new block (or function call) is one level higher, and so on.
    pub scope_level: usize,
    /// Force a [`Scope`][crate::Scope] search by name.
    ///
    /// Normally, access to variables are parsed with a relative offset into the
    /// [`Scope`][crate::Scope] to avoid a lookup.
    ///
    /// In some situation, e.g. after running an `eval` statement, or after a custom syntax
    /// statement, subsequent offsets may become mis-aligned.
    ///
    /// When that happens, this flag is turned on.
    pub always_search_scope: bool,
    /// Function call hashes to index getters and setters.
    #[cfg(any(not(feature = "no_index"), not(feature = "no_object")))]
    fn_hash_indexing: (u64, u64),
    /// Embedded [module][crate::Module] resolver.
    #[cfg(not(feature = "no_module"))]
    pub embedded_module_resolver:
        Option<crate::Shared<crate::module::resolvers::StaticModuleResolver>>,
    /// Cache of globally-defined constants.
    ///
    /// Interior mutability is needed because it is shared in order to aid in cloning.
    #[cfg(not(feature = "no_module"))]
    #[cfg(not(feature = "no_function"))]
    pub constants: Option<GlobalConstants>,
    /// Custom state that can be used by the external host.
    pub tag: Dynamic,
    /// Debugging interface.
    #[cfg(feature = "debugging")]
    pub debugger: super::Debugger,
}

impl GlobalRuntimeState {
    /// Create a new [`GlobalRuntimeState`] based on an [`Engine`].
    #[inline(always)]
    #[must_use]
    pub fn new(engine: &Engine) -> Self {
        Self {
            #[cfg(not(feature = "no_module"))]
            imports: StaticVec::new_const(),
            #[cfg(not(feature = "no_module"))]
            modules: StaticVec::new_const(),
            lib: StaticVec::new_const(),
            source: None,
            num_operations: 0,
            #[cfg(not(feature = "no_module"))]
            num_modules_loaded: 0,
            scope_level: 0,
            level: 0,
            always_search_scope: false,
            #[cfg(not(feature = "no_module"))]
            embedded_module_resolver: None,
            #[cfg(any(not(feature = "no_index"), not(feature = "no_object")))]
            fn_hash_indexing: (0, 0),
            #[cfg(not(feature = "no_module"))]
            #[cfg(not(feature = "no_function"))]
            constants: None,

            tag: engine.default_tag().clone(),

            #[cfg(feature = "debugging")]
            debugger: crate::eval::Debugger::new(
                if engine.debugger.is_some() {
                    crate::eval::DebuggerStatus::Init
                } else {
                    crate::eval::DebuggerStatus::CONTINUE
                },
                match engine.debugger {
                    Some((ref init, ..)) => init(engine),
                    None => Dynamic::UNIT,
                },
            ),
        }
    }
    /// Get the length of the stack of globally-imported [modules][crate::Module].
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    #[must_use]
    pub fn num_imports(&self) -> usize {
        self.modules.len()
    }
    /// Get the globally-imported [module][crate::Module] at a particular index.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    #[must_use]
    pub fn get_shared_import(&self, index: usize) -> Option<SharedModule> {
        self.modules.get(index).cloned()
    }
    /// Get a mutable reference to the globally-imported [module][crate::Module] at a
    /// particular index.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[allow(dead_code)]
    #[inline(always)]
    #[must_use]
    pub(crate) fn get_shared_import_mut(&mut self, index: usize) -> Option<&mut SharedModule> {
        self.modules.get_mut(index)
    }
    /// Get the index of a globally-imported [module][crate::Module] by name.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    #[must_use]
    pub fn find_import(&self, name: &str) -> Option<usize> {
        self.imports
            .iter()
            .rev()
            .position(|key| key.as_str() == name)
            .map(|i| self.imports.len() - 1 - i)
    }
    /// Push an imported [module][crate::Module] onto the stack.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    pub fn push_import(
        &mut self,
        name: impl Into<ImmutableString>,
        module: impl Into<SharedModule>,
    ) {
        self.imports.push(name.into());
        self.modules.push(module.into());
    }
    /// Truncate the stack of globally-imported [modules][crate::Module] to a particular length.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    pub fn truncate_imports(&mut self, size: usize) {
        self.imports.truncate(size);
        self.modules.truncate(size);
    }
    /// Get an iterator to the stack of globally-imported [modules][crate::Module] in reverse order.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    pub fn iter_imports(&self) -> impl Iterator<Item = (&str, &crate::Module)> {
        self.imports
            .iter()
            .zip(self.modules.iter())
            .rev()
            .map(|(name, module)| (name.as_str(), &**module))
    }
    /// Get an iterator to the stack of globally-imported [modules][crate::Module] in reverse order.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    pub(crate) fn iter_imports_raw(
        &self,
    ) -> impl Iterator<Item = (&ImmutableString, &SharedModule)> {
        self.imports.iter().zip(self.modules.iter()).rev()
    }
    /// Get an iterator to the stack of globally-imported [modules][crate::Module] in forward order.
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    pub fn scan_imports_raw(&self) -> impl Iterator<Item = (&ImmutableString, &SharedModule)> {
        self.imports.iter().zip(self.modules.iter())
    }
    /// Can the particular function with [`Dynamic`] parameter(s) exist in the stack of
    /// globally-imported [modules][crate::Module]?
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    pub(crate) fn may_contain_dynamic_fn(&self, hash_script: u64) -> bool {
        self.modules
            .iter()
            .any(|m| m.may_contain_dynamic_fn(hash_script))
    }
    /// Does the specified function hash key exist in the stack of globally-imported
    /// [modules][crate::Module]?
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub fn contains_qualified_fn(&self, hash: u64) -> bool {
        self.modules.iter().any(|m| m.contains_qualified_fn(hash))
    }
    /// Get the specified function via its hash key from the stack of globally-imported
    /// [modules][crate::Module].
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    #[must_use]
    pub fn get_qualified_fn(
        &self,
        hash: u64,
    ) -> Option<(&crate::func::CallableFunction, Option<&ImmutableString>)> {
        self.modules
            .iter()
            .rev()
            .find_map(|m| m.get_qualified_fn(hash).map(|f| (f, m.id_raw())))
    }
    /// Does the specified [`TypeId`][std::any::TypeId] iterator exist in the stack of
    /// globally-imported [modules][crate::Module]?
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub fn contains_iter(&self, id: std::any::TypeId) -> bool {
        self.modules.iter().any(|m| m.contains_qualified_iter(id))
    }
    /// Get the specified [`TypeId`][std::any::TypeId] iterator from the stack of globally-imported
    /// [modules][crate::Module].
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline]
    #[must_use]
    pub fn get_iter(&self, id: std::any::TypeId) -> Option<&crate::func::IteratorFn> {
        self.modules
            .iter()
            .rev()
            .find_map(|m| m.get_qualified_iter(id))
    }
    /// Get the current source.
    #[inline(always)]
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_ref().map(|s| s.as_str())
    }
    /// Get the current source.
    #[inline(always)]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn source_raw(&self) -> Option<&ImmutableString> {
        self.source.as_ref()
    }
    /// Get the pre-calculated index getter hash.
    #[cfg(any(not(feature = "no_index"), not(feature = "no_object")))]
    #[must_use]
    pub(crate) fn hash_idx_get(&mut self) -> u64 {
        if self.fn_hash_indexing == (0, 0) {
            let n1 = crate::calc_fn_hash(None, crate::engine::FN_IDX_GET, 2);
            let n2 = crate::calc_fn_hash(None, crate::engine::FN_IDX_SET, 3);
            self.fn_hash_indexing = (n1, n2);
            n1
        } else {
            self.fn_hash_indexing.0
        }
    }
    /// Get the pre-calculated index setter hash.
    #[cfg(any(not(feature = "no_index"), not(feature = "no_object")))]
    #[must_use]
    pub(crate) fn hash_idx_set(&mut self) -> u64 {
        if self.fn_hash_indexing == (0, 0) {
            let n1 = crate::calc_fn_hash(None, crate::engine::FN_IDX_GET, 2);
            let n2 = crate::calc_fn_hash(None, crate::engine::FN_IDX_SET, 3);
            self.fn_hash_indexing = (n1, n2);
            n2
        } else {
            self.fn_hash_indexing.1
        }
    }
}

#[cfg(not(feature = "no_module"))]
impl<K: Into<ImmutableString>, M: Into<SharedModule>> Extend<(K, M)> for GlobalRuntimeState {
    #[inline]
    fn extend<T: IntoIterator<Item = (K, M)>>(&mut self, iter: T) {
        for (k, m) in iter {
            self.imports.push(k.into());
            self.modules.push(m.into());
        }
    }
}

impl fmt::Debug for GlobalRuntimeState {
    #[cold]
    #[inline(never)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("GlobalRuntimeState");

        #[cfg(not(feature = "no_module"))]
        f.field("imports", &self.scan_imports_raw().collect::<Vec<_>>());

        f.field("source", &self.source)
            .field("num_operations", &self.num_operations);

        #[cfg(any(not(feature = "no_index"), not(feature = "no_object")))]
        f.field("fn_hash_indexing", &self.fn_hash_indexing);

        #[cfg(not(feature = "no_module"))]
        f.field("num_modules_loaded", &self.num_modules_loaded)
            .field("embedded_module_resolver", &self.embedded_module_resolver);

        #[cfg(not(feature = "no_module"))]
        #[cfg(not(feature = "no_function"))]
        f.field("constants", &self.constants);

        f.finish()
    }
}
