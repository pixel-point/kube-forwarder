//! Reading kube configs and building Kubernetes clients for a cluster.

use std::path::{Path, PathBuf};

use kube::config::{KubeConfigOptions, Kubeconfig};

use crate::error::{KfError, Result, error_chain, kube_error};
use crate::model::{ClusterConfig, StoringMethod};

/// The original application refused to copy config files bigger than 1 MB.
pub const MAX_CONFIG_FILE_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextInfo {
    pub name: String,
    pub cluster: String,
    pub user: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedKubeconfig {
    pub kubeconfig: Kubeconfig,
    pub contexts: Vec<ContextInfo>,
    pub current_context: Option<String>,
}

impl ParsedKubeconfig {
    fn new(kubeconfig: Kubeconfig) -> Self {
        let contexts = kubeconfig
            .contexts
            .iter()
            .map(|named| ContextInfo {
                name: named.name.clone(),
                cluster: named
                    .context
                    .as_ref()
                    .map(|context| context.cluster.clone())
                    .unwrap_or_default(),
                user: named
                    .context
                    .as_ref()
                    .and_then(|context| context.user.clone()),
                namespace: named
                    .context
                    .as_ref()
                    .and_then(|context| context.namespace.clone()),
            })
            .collect();
        let current_context = kubeconfig
            .current_context
            .clone()
            .filter(|name| !name.is_empty());
        Self {
            kubeconfig,
            contexts,
            current_context,
        }
    }

    pub fn has_context(&self, name: &str) -> bool {
        self.contexts.iter().any(|context| context.name == name)
    }

    pub fn context_names(&self) -> Vec<String> {
        self.contexts
            .iter()
            .map(|context| context.name.clone())
            .collect()
    }
}

/// Expands a leading `~/` to the home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(path)
}

pub fn read_file(path: &Path) -> Result<ParsedKubeconfig> {
    if !path.exists() {
        return Err(KfError::new(format!(
            "File {} doesn't exist.",
            path.display()
        )));
    }
    let kubeconfig = Kubeconfig::read_from(path).map_err(|error| {
        KfError::new(format!(
            "Failed to read the kube config {}.",
            path.display()
        ))
        .with_details(error_chain(&error))
    })?;
    Ok(ParsedKubeconfig::new(kubeconfig))
}

pub fn parse_str(text: &str) -> Result<ParsedKubeconfig> {
    let kubeconfig = Kubeconfig::from_yaml(text).map_err(|error| {
        KfError::new("The config is not a valid kube config.").with_details(error_chain(&error))
    })?;
    Ok(ParsedKubeconfig::new(kubeconfig))
}

/// Reads a (small) text file, used to copy a kube config into the form.
pub fn read_text_file(path: &Path) -> Result<String> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        KfError::new(format!("Failed to open {}.", path.display())).with_details(error.to_string())
    })?;
    if metadata.len() > MAX_CONFIG_FILE_SIZE {
        return Err(KfError::new("Sorry, the file is too large (> 1MB)."));
    }
    let bytes = std::fs::read(path).map_err(|error| {
        KfError::new(format!("Failed to read {}.", path.display())).with_details(error.to_string())
    })?;
    let text =
        String::from_utf8(bytes).map_err(|_| KfError::new("The file is not a UTF-8 text file."))?;
    Ok(text.trim_start_matches('\u{feff}').to_string())
}

/// `~/.kube/config`
pub fn default_kubeconfig_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".kube").join("config"))
}

/// Loads the kube config referenced by a cluster and the name of the context to use.
pub fn load(config: &ClusterConfig) -> Result<(Kubeconfig, Option<String>)> {
    match config.storing_method {
        StoringMethod::Path => {
            let path = config
                .path
                .as_deref()
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .ok_or_else(|| KfError::new("The path to the kube config is empty."))?;
            let parsed = read_file(&expand_tilde(path))?;
            let context = config
                .current_context
                .clone()
                .filter(|context| !context.is_empty())
                .or_else(|| parsed.current_context.clone());
            if let Some(context) = &context
                && !parsed.has_context(context)
            {
                return Err(KfError::new(format!(
                    "Context \"{context}\" was not found in {path}."
                )));
            }
            Ok((parsed.kubeconfig, context))
        }
        StoringMethod::Content => {
            let content = config
                .content
                .as_deref()
                .filter(|content| !content.trim().is_empty())
                .ok_or_else(|| KfError::new("The kube config is empty."))?;
            let parsed = parse_str(content)?;
            // The original application always used `current-context` of a pasted config.
            // A stored context is preferred when it exists (exports of "path" clusters keep
            // the context that was selected in the application).
            let context = config
                .current_context
                .clone()
                .filter(|context| parsed.has_context(context))
                .or_else(|| parsed.current_context.clone());
            Ok((parsed.kubeconfig, context))
        }
    }
}

/// Builds a Kubernetes client for a cluster.
pub async fn client_for(config: &ClusterConfig) -> Result<kube::Client> {
    let (kubeconfig, context) = load(config)?;
    check_exec_command(&kubeconfig, context.as_deref())?;
    let options = KubeConfigOptions {
        context,
        ..Default::default()
    };
    let config = kube::Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .map_err(|error| {
            KfError::new("Cluster config is invalid.").with_details(error_chain(&error))
        })?;
    kube::Client::try_from(config).map_err(|error| kube_error(&error, None))
}

/// Exec credential plugins (aws, gcloud, kubelogin, ...) are a common source of confusing
/// errors when they are not installed or not on `PATH`. Check them upfront.
fn check_exec_command(kubeconfig: &Kubeconfig, context: Option<&str>) -> Result<()> {
    let context_name = context.or(kubeconfig.current_context.as_deref());
    let Some(context) = kubeconfig
        .contexts
        .iter()
        .find(|named| Some(named.name.as_str()) == context_name)
        .and_then(|named| named.context.as_ref())
    else {
        return Ok(());
    };
    let Some(user) = context.user.as_deref() else {
        return Ok(());
    };
    let Some(command) = kubeconfig
        .auth_infos
        .iter()
        .find(|named| named.name == user)
        .and_then(|named| named.auth_info.as_ref())
        .and_then(|auth| auth.exec.as_ref())
        .and_then(|exec| exec.command.as_deref())
    else {
        return Ok(());
    };

    if command_exists(command) {
        Ok(())
    } else {
        Err(missing_command_error(command))
    }
}

fn command_exists(command: &str) -> bool {
    let path = Path::new(command);
    if path.is_absolute() || path.components().count() > 1 {
        return is_executable(path);
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(command))))
        .unwrap_or(false)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file() || path.with_extension("exe").is_file()
}

fn missing_command_error(command: &str) -> KfError {
    let name = Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command);
    let hint = match name {
        "aws" => "Please make sure you have installed AWS CLI. (https://docs.aws.amazon.com/cli/)",
        "aws-iam-authenticator" => {
            "Please make sure you have installed aws-iam-authenticator. \
             (https://github.com/kubernetes-sigs/aws-iam-authenticator)"
        }
        "gcloud" | "gke-gcloud-auth-plugin" => {
            "Please make sure you have installed Google Cloud SDK and gke-gcloud-auth-plugin. \
             (https://cloud.google.com/sdk)"
        }
        "kubelogin" => "Please make sure you have installed kubelogin.",
        "kubectl" => {
            "Please make sure you have installed kubectl and the plugin used by this config."
        }
        _ => "Please make sure the command is installed.",
    };
    let path = std::env::var("PATH").unwrap_or_default();
    KfError::new(format!(
        "Command \"{command}\" required to authenticate with this cluster was not found."
    ))
    .with_details(format!("{hint}\n\nPATH: {path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const CONFIG: &str = r#"
apiVersion: v1
kind: Config
current-context: first
clusters:
- name: one
  cluster:
    server: https://127.0.0.1:6443
    insecure-skip-tls-verify: true
- name: two
  cluster:
    server: https://127.0.0.2:6443
contexts:
- name: first
  context:
    cluster: one
    user: admin
    namespace: apps
- name: second
  context:
    cluster: two
    user: missing-plugin
users:
- name: admin
  user:
    token: abc
- name: missing-plugin
  user:
    exec:
      apiVersion: client.authentication.k8s.io/v1beta1
      command: definitely-not-installed-kf-plugin
"#;

    #[test]
    fn parses_contexts() {
        let parsed = parse_str(CONFIG).unwrap();
        assert_eq!(parsed.current_context.as_deref(), Some("first"));
        assert_eq!(parsed.context_names(), ["first", "second"]);
        assert_eq!(parsed.contexts[0].cluster, "one");
        assert_eq!(parsed.contexts[0].user.as_deref(), Some("admin"));
        assert_eq!(parsed.contexts[0].namespace.as_deref(), Some("apps"));
    }

    #[test]
    fn invalid_yaml_is_reported() {
        let error = parse_str("clusters: [").unwrap_err();
        assert_eq!(error.message, "The config is not a valid kube config.");
        assert!(error.details.is_some());
    }

    #[test]
    fn content_uses_stored_context_when_it_exists() {
        let (_, context) =
            load(&ClusterConfig::from_content(CONFIG, Some("second".into()))).unwrap();
        assert_eq!(context.as_deref(), Some("second"));
        let (_, context) = load(&ClusterConfig::from_content(CONFIG, Some("nope".into()))).unwrap();
        assert_eq!(context.as_deref(), Some("first"));
        let (_, context) = load(&ClusterConfig::from_content(CONFIG, None)).unwrap();
        assert_eq!(context.as_deref(), Some("first"));
    }

    #[test]
    fn path_requires_an_existing_context() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config");
        std::fs::write(&path, CONFIG).unwrap();
        let path_str = path.to_str().unwrap();

        let (_, context) = load(&ClusterConfig::from_path(path_str, "second")).unwrap();
        assert_eq!(context.as_deref(), Some("second"));

        let error = load(&ClusterConfig::from_path(path_str, "nope")).unwrap_err();
        assert!(error.message.contains("Context \"nope\" was not found"));

        let error = load(&ClusterConfig::from_path("/definitely/missing", "a")).unwrap_err();
        assert!(error.message.contains("doesn't exist"));
    }

    #[tokio::test]
    async fn missing_exec_plugin_is_reported_before_connecting() {
        let Err(error) =
            client_for(&ClusterConfig::from_content(CONFIG, Some("second".into()))).await
        else {
            panic!("a client must not be built when the exec plugin is missing");
        };
        assert!(
            error.message.contains("definitely-not-installed-kf-plugin"),
            "{error:?}"
        );
    }

    #[tokio::test]
    async fn builds_a_client_for_token_auth() {
        assert!(
            client_for(&ClusterConfig::from_content(CONFIG, None))
                .await
                .is_ok()
        );
    }

    #[test]
    fn read_text_file_limits_size() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big");
        std::fs::write(&big, vec![b'a'; (MAX_CONFIG_FILE_SIZE + 1) as usize]).unwrap();
        assert!(
            read_text_file(&big)
                .unwrap_err()
                .message
                .contains("too large")
        );

        let small = dir.path().join("small");
        std::fs::write(&small, "\u{feff}apiVersion: v1").unwrap();
        assert_eq!(read_text_file(&small).unwrap(), "apiVersion: v1");
    }
}
