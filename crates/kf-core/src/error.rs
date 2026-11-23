//! User facing errors.
//!
//! Everything that can go wrong and should be shown to a user is converted into a
//! [`KfError`]: a short, readable message plus optional technical details (the
//! original app showed those behind a "Details" button).

use std::fmt;

/// A user facing error: a short message and optional technical details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KfError {
    pub message: String,
    pub details: Option<String>,
    cancelled: bool,
}

pub type Result<T, E = KfError> = std::result::Result<T, E>;

impl KfError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: None,
            cancelled: false,
        }
    }

    /// An operation that was cancelled by the user. It should not be reported.
    pub fn cancelled() -> Self {
        Self {
            message: "Cancelled".into(),
            details: None,
            cancelled: true,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        let details = details.into();
        let details = details.trim();
        if !details.is_empty() && details != self.message {
            self.details = Some(details.to_string());
        }
        self
    }

    /// Message and details in one string.
    pub fn full_text(&self) -> String {
        match &self.details {
            Some(details) => format!("{}\n\n{}", self.message, details),
            None => self.message.clone(),
        }
    }
}

impl fmt::Display for KfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for KfError {}

impl From<std::io::Error> for KfError {
    fn from(error: std::io::Error) -> Self {
        KfError::new(error.to_string())
    }
}

/// Joins an error and all of its sources into one line, skipping repeated parts.
pub fn error_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(err) = current {
        let text = err.to_string();
        if !parts.iter().any(|part| part.contains(&text)) {
            parts.push(text);
        }
        current = err.source();
    }
    parts.join(": ")
}

/// Converts a Kubernetes client error into a readable error.
///
/// `object` is a description of the requested object (for example `Pod "postgres"`), it
/// mirrors the `k8nApiPrettyError` helper of the original application.
pub fn kube_error(error: &kube::Error, object: Option<&str>) -> KfError {
    match error {
        kube::Error::Api(status) => {
            let postfix = match status.code {
                404 => "not found.",
                403 => "forbidden.",
                401 => "unauthorized.",
                _ => "can't be fetched.",
            };
            let message = match object {
                Some(object) => format!("{object} {postfix}"),
                None if !status.message.is_empty() => status.message.clone(),
                None => match status.code {
                    401 => "Unauthorized. Please check the credentials in your kube config.".into(),
                    403 => "Forbidden.".into(),
                    _ => format!("The cluster responded with an error ({}).", status.code),
                },
            };
            KfError::new(message).with_details(status.message.clone())
        }
        kube::Error::Auth(auth) => {
            KfError::new("Failed to authenticate with the cluster.").with_details(error_chain(auth))
        }
        other => {
            let message = match object {
                Some(object) => format!("{object} can't be fetched."),
                None => "Failed to connect to the cluster.".into(),
            };
            KfError::new(message).with_details(error_chain(other))
        }
    }
}

/// Converts errors of a local TCP listener into readable errors.
pub fn bind_error(error: &std::io::Error, address: &str, port: u16) -> KfError {
    use std::io::ErrorKind;

    match error.kind() {
        ErrorKind::AddrInUse => KfError::new(format!("Port {address}:{port} is already in use.")),
        ErrorKind::PermissionDenied => KfError::new(
            "Kube Forwarder doesn't have enough privileges to use local ports below 1024.",
        ),
        ErrorKind::AddrNotAvailable => {
            let mut err = KfError::new(format!(
                "Address {address} is not available on this machine."
            ));
            if cfg!(target_os = "macos") && address.starts_with("127.") && address != "127.0.0.1" {
                err = err.with_details(format!(
                    "macOS only configures 127.0.0.1 on the loopback interface. \
                     Add an alias first:\n\nsudo ifconfig lo0 alias {address} up"
                ));
            }
            err
        }
        _ => KfError::new(format!("Failed to listen on {address}:{port}."))
            .with_details(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::Status;

    fn api_error(code: u16, message: &str) -> kube::Error {
        let mut status = Status::failure(message, "");
        status.code = code;
        kube::Error::Api(Box::new(status))
    }

    #[test]
    fn api_errors_with_object() {
        let err = kube_error(&api_error(404, "pods \"x\" not found"), Some("Pod \"x\""));
        assert_eq!(err.message, "Pod \"x\" not found.");
        assert_eq!(err.details.as_deref(), Some("pods \"x\" not found"));

        let err = kube_error(&api_error(403, "forbidden: user"), Some("Service \"s\""));
        assert_eq!(err.message, "Service \"s\" forbidden.");

        let err = kube_error(&api_error(500, "boom"), Some("Deployment \"d\""));
        assert_eq!(err.message, "Deployment \"d\" can't be fetched.");
    }

    #[test]
    fn api_errors_without_object_use_server_message() {
        let err = kube_error(&api_error(403, "nodes is forbidden"), None);
        assert_eq!(err.message, "nodes is forbidden");
        assert_eq!(err.details, None);
    }

    #[test]
    fn bind_errors_are_readable() {
        let in_use = std::io::Error::from(std::io::ErrorKind::AddrInUse);
        assert_eq!(
            bind_error(&in_use, "localhost", 8080).message,
            "Port localhost:8080 is already in use."
        );
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert!(
            bind_error(&denied, "localhost", 80)
                .message
                .contains("below 1024")
        );
    }

    #[test]
    fn details_equal_to_message_are_dropped() {
        let err = KfError::new("same").with_details("same");
        assert_eq!(err.details, None);
        assert_eq!(err.full_text(), "same");
    }
}
