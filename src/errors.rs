
macro_rules! error {
    ($errors:ident, $loc:expr, $($msg:tt)+) => {
        {
            let msg = format!($($msg)+);
            let int_loc = InternalLocation {
                file: file!(),
                line: line!(),
            };
            push_error($errors, &int_loc, &$loc, msg);
        }
    };
}

index_type!(SourceFile);

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

    pub fn file(&self) -> SourceFile { self.file }
    pub fn begin(&self) -> usize { self.begin as usize }
    pub fn end(&self) -> usize { self.end as usize }
}

pub trait Errors {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>);
}

pub fn push_error(errors: &mut impl Errors, int_loc: &InternalLocation, loc: &impl Locatable, message: String) {
    errors.push(int_loc, loc, message, Vec::new());
}

#[derive(Clone)]
pub struct InternalLocation {
    pub file: &'static str,
    pub line: u32,
}

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
