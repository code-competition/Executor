use std::fmt::Display;

use bollard::models::ContainerWaitResponseError;

#[derive(Debug, Clone)]
pub struct Error {
    #[allow(dead_code)]
    pub(crate) status_code: i64,
    pub(crate) error: Option<ContainerWaitResponseError>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let el = self.error.as_ref().unwrap();
        let el = el.message.as_ref().unwrap();
        f.write_str(el)
    }
}

impl std::error::Error for Error {}
