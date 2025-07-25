use std::path::Path;

use actix_http::Response;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    body::{BodyStream, BoxBody, to_bytes_limited},
    dev::{Payload, ServiceRequest},
    http::{StatusCode, Version, header},
};

use crate::{builder::Builder, error::Error, factory::Middleware};

const CONNECTION_INFO: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Default)]
struct TransactionConfig {
    max_request_body: Option<usize>,
    max_response_body: Option<usize>,
}

/// Actix-Web compatible wrapper on [`ModSecurity`](modsecurity::ModSecurity)
pub struct ModSecurity {
    config: TransactionConfig,
    rules: modsecurity::Rules,
    security: modsecurity::ModSecurity,
}

impl ModSecurity {
    /// Creates a new [`ModSecurity`](crate::ModSecurity) instance.
    ///
    /// Because of implementation specifics of LibModSecurity, it is
    /// recommended only once instance exist within the program.
    ///
    /// See [`modsecurity::msc::ModSecurity`](modsecurity::msc::ModSecurity)
    /// for more details.
    pub fn new() -> Self {
        Self {
            config: TransactionConfig::default(),
            rules: modsecurity::Rules::new(),
            security: modsecurity::ModSecurity::builder()
                .with_connector_info(CONNECTION_INFO)
                .expect("failed to add connector into")
                .build(),
        }
    }

    /// Generate [`Builder`] for [`ModSecurity`] instance.
    #[inline]
    pub fn builder() -> Builder {
        Self::new().into()
    }

    /// Adds plain rules from string into the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_modsecurity::ModSecurity;
    ///
    /// let mut security = ModSecurity::new();
    /// security.add_rules("SecRuleEngine On\n").expect("Failed to add rules");
    /// ```
    pub fn add_rules(&mut self, rules: &str) -> Result<&mut Self, Error> {
        self.rules.add_plain(rules)?;
        Ok(self)
    }

    /// Adds rules from a file into the set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use actix_modsecurity::ModSecurity;
    ///
    /// let mut security = ModSecurity::new();
    /// security.add_rules_file("/path/to/rules.conf").expect("Failed to add rules from file");
    /// ```
    pub fn add_rules_file<P: AsRef<Path>>(&mut self, file: P) -> Result<&mut Self, Error> {
        self.rules.add_file(file)?;
        Ok(self)
    }

    /// Configure Max request body size allowed to be loaded into memory for processing.
    ///
    /// This avoids out-of-memory errors and potential security-risks from attackers
    /// overloading your web-service.
    pub fn set_max_request_size(&mut self, max_request_body: Option<usize>) -> &mut Self {
        self.config.max_request_body = max_request_body;
        self
    }

    /// Configure Max response body size allowed to be loaded into memory for processing.
    ///
    /// This avoids out-of-memory errors and potential security-risks from attackers
    /// overloading your web-service.
    pub fn set_max_response_size(&mut self, max_response_body: Option<usize>) -> &mut Self {
        self.config.max_response_body = max_response_body;
        self
    }

    /// Creates a configured LibModSecurity Transaction with the configured rules.
    pub fn transaction(&self) -> Result<Transaction, Error> {
        Ok(Transaction {
            config: self.config.clone(),
            transaction: self
                .security
                .transaction_builder()
                .with_rules(&self.rules)
                .build()?,
        })
    }

    /// Converts ModSecurity Instance into Actix-Web Middleware
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_web::App;
    /// use actix_modsecurity::ModSecurity;
    ///
    /// let mut security = ModSecurity::new();
    /// security.add_rules("SecRuleEngine On\n").expect("Failed to add rules");
    ///
    /// let app = App::new()
    ///     .wrap(security.middleware());
    /// ```
    #[inline]
    pub fn middleware(self) -> Middleware {
        self.into()
    }
}

impl Default for ModSecurity {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn version_str(v: Version) -> &'static str {
    match v {
        Version::HTTP_09 => "0.9",
        Version::HTTP_10 => "1.0",
        Version::HTTP_11 => "1.1",
        Version::HTTP_2 => "2",
        Version::HTTP_3 => "3",
        _ => panic!("unexpected http version!"),
    }
}

#[inline]
fn intervention_response(intv: &modsecurity::Intervention) -> Result<HttpResponse, Error> {
    if let Some(log) = intv.log() {
        tracing::error!("{log}");
    }
    if let Some(url) = intv.url() {
        let mut res = HttpResponse::TemporaryRedirect();
        res.insert_header((header::LOCATION, url));
        return Ok(res.into());
    }
    let code = StatusCode::from_u16(intv.status() as u16)?;
    Ok(HttpResponse::new(code))
}

/// Actix-Web compatible wrapper on [`Tranaction`](modsecurity::Transaction)
pub struct Transaction<'a> {
    config: TransactionConfig,
    transaction: modsecurity::Transaction<'a>,
}

impl<'a> Transaction<'a> {
    /// Performs analysis on the connection.
    ///
    /// This should be called at the very beginning of a request process.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub fn process_connection(&mut self, req: &HttpRequest) -> Result<(), Error> {
        let Some(caddr) = req.peer_addr() else {
            tracing::warn!("missing client-address. cannot scan connection");
            return Ok(());
        };

        let saddr = req.app_config().local_addr();
        Ok(self.transaction.process_connection(
            &caddr.ip().to_string(),
            caddr.port() as i32,
            &saddr.ip().to_string(),
            saddr.port() as i32,
        )?)
    }

    /// Perform the analysis on the URI and all the query string variables.
    ///
    /// This should be called at the very beginning of a request process.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    #[inline]
    pub fn process_uri(&mut self, req: &HttpRequest) -> Result<(), Error> {
        Ok(self.transaction.process_uri(
            &req.uri().to_string(),
            req.method().as_str(),
            version_str(req.version()),
        )?)
    }

    /// Processes rules in the request headers phase for this transaction.
    ///
    /// This should be called at the very beginning of a request process.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    #[inline]
    pub fn process_request_headers(&mut self, req: &HttpRequest) -> Result<(), Error> {
        req.headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str(), v.to_str().ok()?)))
            .try_for_each(|(k, v)| self.transaction.add_request_header(k, v))?;
        Ok(self.transaction.process_request_headers()?)
    }

    /// Processes rules in the request body phase for this transaction.
    ///
    /// This should be called at the very beginning of a request process.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub async fn process_request_body(&mut self, payload: Payload) -> Result<Payload, Error> {
        let max = self.config.max_request_body.unwrap_or(u16::MAX as usize);
        let stream = BodyStream::new(payload);
        let body = to_bytes_limited(stream, max).await??;
        self.transaction.append_request_body(&body)?;
        self.transaction.process_request_body()?;

        let (_, mut payload) = actix_http::h1::Payload::create(true);
        payload.unread_data(body);
        Ok(Payload::H1 { payload })
    }

    /// Processes *ALL* rules in the request phase for this transaction.
    ///
    /// This should be called at the very beginning of a request process.
    /// Use this instead of any of the following:
    ///
    ///  - [`Transaction::process_connection`]
    ///  - [`Transaction::process_uri`]
    ///  - [`Transaction::process_request_headers`]
    ///  - [`Transaction::process_request_body`]
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub async fn process_request(&mut self, req: &mut ServiceRequest) -> Result<(), Error> {
        self.process_connection(req.request())?;
        self.process_uri(req.request())?;
        self.process_request_headers(req.request())?;
        let payload = self.process_request_body(req.take_payload()).await?;
        req.set_payload(payload);
        Ok(())
    }

    /// Processes rules in the response headers phase for this transaction.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub fn process_response_headers<T>(&mut self, res: &HttpResponse<T>) -> Result<(), Error> {
        let code: u16 = res.status().into();
        let version = format!("HTTP {}", version_str(res.head().version));
        res.headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str(), v.to_str().ok()?)))
            .try_for_each(|(k, v)| self.transaction.add_response_header(k, v))?;
        Ok(self
            .transaction
            .process_response_headers(code as i32, &version)?)
    }

    /// Processes rules in the response body phase for this transaction.
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub async fn process_response_body(&mut self, body: BoxBody) -> Result<BoxBody, Error> {
        let max = self.config.max_response_body.unwrap_or(u16::MAX as usize);
        let body = to_bytes_limited(body, max).await??;
        self.transaction.append_response_body(&body)?;
        self.transaction.process_response_body()?;
        Ok(BoxBody::new(body))
    }

    /// Processes *ALL* rules in the response phase for this transaction.
    ///
    /// This should be called at the very beginning of a request process.
    /// Use this instead of any of the following:
    ///
    ///  - [`Transaction::process_response_headers`]
    ///  - [`Transaction::process_response_body`]
    ///
    /// **NOTE**: Remember to check for a possible intervention using
    /// [`Transaction::intervention()`] after calling this method.
    pub async fn process_response(&mut self, res: HttpResponse) -> Result<HttpResponse, Error> {
        let (http_res, mut body) = res.into_parts();
        self.process_response_headers(&http_res)?;
        body = self.process_response_body(body).await?;
        Ok(http_res.set_body(body))
    }

    /// Returns an intervention if one is triggered by the transaction.
    ///
    /// An intervention is triggered when a rule is matched and the
    /// corresponding action is disruptive.
    pub fn intervention(&mut self) -> Result<Option<Intervention>, Error> {
        let Some(intv) = self.transaction.intervention() else {
            return Ok(None);
        };
        let response = intervention_response(&intv)?;
        Ok(Some(Intervention {
            message: intv.log().map(|s| s.to_owned()),
            url: intv.url().map(|u| u.to_owned()),
            code: StatusCode::from_u16(intv.status() as u16)?,
            response,
        }))
    }
}

/// Actix-Web compatible wrapper on
/// [`Intervention`](modsecurity::intervention::Intervention)
#[derive(Debug)]
pub struct Intervention {
    message: Option<String>,
    url: Option<String>,
    code: StatusCode,
    response: HttpResponse<BoxBody>,
}

impl Intervention {
    /// Returns the log message, if any, of the intervention.
    #[inline]
    pub fn log(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns the URL, if any, of the intervention.
    #[inline]
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Returns the status code of the intervention.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.code
    }

    /// Returns the repacement HttpResponse of the intervention
    pub fn response(&self) -> &HttpResponse {
        &self.response
    }
}

impl From<Intervention> for HttpResponse {
    fn from(value: Intervention) -> Self {
        value.response
    }
}

impl From<Intervention> for Response<BoxBody> {
    fn from(value: Intervention) -> Self {
        value.response.into()
    }
}
