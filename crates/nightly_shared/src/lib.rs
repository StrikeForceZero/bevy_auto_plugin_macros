#![feature(proc_macro_span)]

pub fn get_file_path() -> String {
    proc_macro2::Span::call_site()
        .unwrap()
        .source_file()
        .path()
        .display()
        .to_string()
}