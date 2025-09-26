use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::errors::*;

#[derive(Default)]
pub struct Compiler {
    errors: Mutex<Vec<ErrorInfo>>,
    files: DashMap<String, SourceFile>,
    file_index: AtomicUsize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler::default()
    }

    pub fn get_file(&self, path: &str) -> SourceFile {
        if let Some(file) = self.files.get(path) {
            return *file;
        };

        *self.files.entry(path.into())
            .or_insert_with(|| SourceFile::new(self.file_index.fetch_add(1, Ordering::Relaxed)))
            .value()
    }
}

impl Locator for Compiler {
}

impl Errors for Compiler {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>) {
        let location = loc.source_span(self);

        let info = ErrorInfo {
            message,
            location,
            internal_location: int_loc.clone(),
            context,
        };

        let mut errors = self.errors.lock();
        errors.push(info);
    }
}
