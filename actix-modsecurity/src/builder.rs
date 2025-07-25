use std::path::Path;

use crate::{error::Error, modsecurity::ModSecurity};

/// Builder abstraction for [`ModSecurity`] object.
///
/// Construct using [`ModSecurity::builder`]
pub struct Builder(ModSecurity);

impl Builder {
    /// Builder equivalent of [`ModSecurity::set_max_request_size`]
    pub fn max_request_size(mut self, max_request_body: Option<usize>) -> Self {
        self.0.set_max_request_size(max_request_body);
        self
    }

    /// Builder equivalent of [`ModSecurity::set_max_response_size`]
    pub fn max_response_size(mut self, max_response_body: Option<usize>) -> Self {
        self.0.set_max_response_size(max_response_body);
        self
    }

    /// Builder equivalent of [`ModSecurity::add_rules`]
    #[inline]
    pub fn rules(mut self, rules: &str) -> Result<Self, Error> {
        self.0.add_rules(rules)?;
        Ok(self)
    }

    /// Builder equivalent of [`ModSecurity::add_rules_file`]
    pub fn rules_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error> {
        self.0.add_rules_file(path)?;
        Ok(self)
    }

    /// Produce built [`ModSecurity`] instance.
    #[inline]
    pub fn build(self) -> ModSecurity {
        self.0
    }
}

impl From<ModSecurity> for Builder {
    #[inline]
    fn from(value: ModSecurity) -> Self {
        Self(value)
    }
}
