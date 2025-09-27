use std::{
    borrow::Cow, cell::OnceCell, collections::HashMap, sync::{
        atomic::{AtomicUsize, Ordering}, Arc
    }
};

use dashmap::DashMap;
use parking_lot::Mutex;

macro_rules! error {
    ($errors:ident, $loc:expr, $($msg:tt)+) => {
        {
            let msg = format!($($msg)+);
            let int_loc = InternalLocation {
                file: file!(),
                line: line!(),
            };
            push_error($errors, &int_loc, $loc, msg);
        }
    };
}

index_type!(SourceFile);

impl SourceFile {
    fn unknown() -> SourceFile {
        SourceFile::new(0)
    }
}

pub struct SourceFileInfo {
    file: SourceFile,
    name: String,
    data: Vec<u8>,
    line_breaks: OnceCell<Vec<u32>>,
}

impl SourceFileInfo {
    pub fn file(&self) -> SourceFile {
        self.file
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn line_breaks(&self) -> &[u32] {
        self.line_breaks
            .get_or_init(|| {
                let line_breaks = self
                    .data
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c == b'\n')
                    .map(|(ix, _)| (ix + 1) as u32);
                std::iter::once(0).chain(line_breaks).collect()
            })
            .as_slice()
    }

    pub fn resolve_line_column(&self, offset: usize) -> (u32, u32) {
        let line_breaks = self.line_breaks();

        let offset = offset as u32;
        let line = match line_breaks.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let line_offset = match line_breaks.get(line) {
            Some(b) => offset - b,
            None => 0,
        };

        let prefix = &self.get_line(line as u32)[..line_offset as usize];
        let col = String::from_utf8_lossy(prefix).chars().count();

        (line as u32, col as u32)
    }

    pub fn get_line(&self, index: u32) -> &[u8] {
        let line_breaks = self.line_breaks();
        let max_len = self.data.len() as u32;
        let begin = line_breaks.get(index as usize).copied().unwrap_or(max_len);
        let end = line_breaks.get((index + 1) as usize).copied().unwrap_or(max_len);
        &self.data[begin as usize..end as usize]
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SourceSpan {
    file: SourceFile,
    begin: u32,
    end: u32,
}

impl SourceSpan {
    pub fn new(file: SourceFile, begin: usize, end: usize) -> SourceSpan {
        SourceSpan {
            file,
            begin: begin as u32,
            end: end as u32,
        }
    }

    pub fn unknown() -> SourceSpan {
        SourceSpan::new(SourceFile::unknown(), 0, 0)
    }

    pub fn file(&self) -> SourceFile {
        self.file
    }
    pub fn begin(&self) -> usize {
        self.begin as usize
    }
    pub fn end(&self) -> usize {
        self.end as usize
    }
}

impl Locatable for SourceSpan {
    fn source_span(&self, _: &dyn Locator) -> SourceSpan {
        *self
    }
}

pub struct SourceSpanInfo {
    span: SourceSpan,
    file_info: Arc<SourceFileInfo>,
    line: u32,
    column: u32,
}

impl SourceSpanInfo {
    pub fn text(&self) -> Cow<'_, str> {
        let data = self.file_info.data();
        String::from_utf8_lossy(&data[self.span.begin()..self.span.end()])
    }

    pub fn filename(&self) -> &str {
        self.file_info.name()
    }

    pub fn line(&self) -> u32 {
        self.line + 1
    }

    pub fn column(&self) -> u32 {
        self.column + 1
    }
}

pub trait Errors {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>);
}

pub fn push_error(errors: &impl Errors, int_loc: &InternalLocation, loc: &impl Locatable, message: String) {
    errors.push(int_loc, loc, message, Vec::new());
}

#[derive(Clone)]
pub struct InternalLocation {
    pub file: &'static str,
    pub line: u32,
}

#[derive(Clone)]
pub struct ErrorInfo {
    pub message: String,
    pub location: SourceSpan,
    pub internal_location: InternalLocation,
    pub context: Vec<String>,
}

pub trait Locator {}

pub trait Locatable {
    fn source_span(&self, locator: &dyn Locator) -> SourceSpan;
}

#[derive(Default)]
pub struct Compiler {
    errors: Mutex<Vec<ErrorInfo>>,
    files_by_name: DashMap<String, Arc<SourceFileInfo>>,
    files_by_file: DashMap<SourceFile, Arc<SourceFileInfo>>,
    file_index: AtomicUsize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            file_index: AtomicUsize::new(1),
            ..Default::default()
        }
    }

    fn load_file_imp(&self, path: &str) -> Arc<SourceFileInfo> {
        let data = match std::fs::read(path) {
            Ok(data) => data,
            Err(err) => {
                error!(self, &SourceSpan::unknown(), "Failed to open file: {path}");
                Vec::new()
            }
        };

        self.add_file(path, data)
    }

    pub fn load_file(&self, path: &str) -> Arc<SourceFileInfo> {
        if let Some(file) = self.files_by_name.get(path) {
            return file.clone();
        };

        self.files_by_name
            .entry(path.into())
            .or_insert_with(|| self.load_file_imp(path))
            .value()
            .clone()
    }

    pub fn add_file(&self, path: &str, source: Vec<u8>) -> Arc<SourceFileInfo> {
        let file = SourceFile::new(self.file_index.fetch_add(1, Ordering::Relaxed));
        let info = SourceFileInfo {
            name: path.to_string(),
            data: source,
            file,
            line_breaks: OnceCell::new(),
        };

        let arc = Arc::new(info);
        self.files_by_file.insert(file, arc.clone());
        arc
    }

    pub fn file_info(&self, file: SourceFile) -> Option<Arc<SourceFileInfo>> {
        self.files_by_file.get(&file).map(|c| c.clone())
    }

    pub fn span_info(&self, span: SourceSpan) -> Option<SourceSpanInfo> {
        if let Some(file_info) = self.file_info(span.file()) {
            let (begin_line, begin_col) = file_info.resolve_line_column(span.begin());

            Some(SourceSpanInfo {
                span,
                file_info,
                line: begin_line,
                column: begin_col,
            })
        } else {
            None
        }
    }

    pub fn errors(&self) -> Vec<ErrorInfo> {
        self.errors.lock().clone()
    }
}

impl Locator for Compiler {}

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
