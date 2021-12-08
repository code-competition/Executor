pub enum Languages {
    Rust,
    Ruby,
    Javascript,
    Python,
}

impl Languages {
    pub const fn get_filename(self) -> &'static str {
        match self {
            Languages::Rust => "main.rs",
            _ => ""
        }
    }
}