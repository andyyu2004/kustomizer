use std::{
    hash::{Hash, Hasher},
    io,
    ops::Deref,
    path::Path,
    sync::LazyLock,
};

use dashmap::DashMap;

#[derive(Debug, Clone, Copy)]
pub struct PathId(&'static Path);

impl AsRef<Path> for PathId {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0
    }
}

impl Deref for PathId {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl PartialEq for PathId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for PathId {}

impl Hash for PathId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const Path).hash(state);
    }
}

static INTERNER: LazyLock<DashMap<&'static Path, PathId>> = LazyLock::new(Default::default);

impl PathId {
    pub fn make(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if let Some(id) = INTERNER.get(path) {
            return Ok(*id);
        }

        let path = path.canonicalize()?;
        if let Some(id) = INTERNER.get(path.as_path()) {
            return Ok(*id);
        }

        let static_path = Box::leak(path.into_boxed_path());
        let id = PathId(static_path);
        // This is not a fully thread-safe implementation, if this actually happens in practice we
        // catch with the assertion below.
        assert!(
            INTERNER.insert(static_path, id).is_none(),
            "PathId already exists in the interner"
        );

        Ok(id)
    }
}
