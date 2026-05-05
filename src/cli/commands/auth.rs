#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, Subcommand};
use is_terminal::IsTerminal;
use serde::Serialize;

use crate::{
    application::{errors::ApplicationError, use_cases::login::LoginUseCase},
    cli::output::{format_json, should_use_json},
    domain::{
        entities::auth_session::CredentialSource,
        errors::AuthError,
        repositories::credential_store::{CredentialStore, StorageKind},
        value_objects::api_key::ApiKey,
    },
    infrastructure::{
        auth::{file_store::FileCredentialStore, keyring_store::KeyringCredentialStore},
        graphql::client::LinearGraphqlClient,
    },
};

#[derive(Args)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub subcommand: AuthSubcommand,
}

#[derive(Subcommand)]
pub enum AuthSubcommand {
    #[command(about = "Store a Linear API key after remote validation")]
    Login {
        #[arg(
            long,
            value_name = "KEY",
            help = "API key (reads from stdin if omitted)"
        )]
        api_key: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            num_args = 0..=1,
            default_missing_value = "",
            help = "Store credential in plain-text file instead of system keychain"
        )]
        store_file: Option<String>,
    },
    #[command(about = "Report authentication status")]
    Status,
    #[command(about = "Remove stored credentials")]
    Logout {
        #[arg(long, help = "Show what would be removed without deleting")]
        dry_run: bool,
    },
}

#[derive(Serialize)]
struct WorkspaceOutput {
    id: String,
    name: String,
    url_key: String,
}

pub async fn run_auth(cmd: &AuthCommand, force_json: bool) -> Result<(), ApplicationError> {
    match &cmd.subcommand {
        AuthSubcommand::Login {
            api_key,
            store_file,
        } => run_login(api_key.as_deref(), store_file.as_deref(), force_json).await,
        AuthSubcommand::Status => run_status(force_json).await,
        AuthSubcommand::Logout { dry_run } => run_logout(*dry_run, force_json).await,
    }
}

fn make_store(
    store_file: Option<&str>,
) -> (Box<dyn CredentialStore>, &'static str, Option<String>) {
    match store_file {
        Some(file_path) => {
            let path = if file_path.is_empty() {
                FileCredentialStore::new().path().to_path_buf()
            } else {
                PathBuf::from(file_path)
            };
            let path_str = path.display().to_string();
            (
                Box::new(FileCredentialStore::with_path(path)),
                "file",
                Some(path_str),
            )
        }
        None => (Box::new(KeyringCredentialStore::new()), "keychain", None),
    }
}

async fn run_login(
    api_key_arg: Option<&str>,
    store_file: Option<&str>,
    force_json: bool,
) -> Result<(), ApplicationError> {
    let api_key = match api_key_arg {
        Some(raw) => ApiKey::new(raw)
            .map_err(|e| ApplicationError::Auth(AuthError::ValidationFailed(e.to_string())))?,
        None => read_api_key(force_json)?,
    };
    let (check_store, storage_kind, storage_path) = make_store(store_file);

    let existing = check_store
        .retrieve()
        .await
        .map_err(ApplicationError::Auth)?;
    let overwrite = if existing.is_some() {
        if should_use_json(force_json) {
            eprintln!("Warning: credential already exists and will be overwritten.");
            true
        } else {
            eprintln!("A credential already exists. Overwrite? [y/N]: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            let answer = input.trim().to_lowercase();
            if answer == "y" || answer == "yes" {
                true
            } else {
                return Err(ApplicationError::Auth(AuthError::Cancelled));
            }
        }
    } else {
        false
    };

    let (fresh_store, _, _) = make_store(store_file);
    let client = Arc::new(LinearGraphqlClient::new());
    let use_case = LoginUseCase::new(client);

    let login_result = use_case
        .execute(api_key, fresh_store, overwrite)
        .await
        .map_err(ApplicationError::Auth)?;

    if should_use_json(force_json) {
        #[derive(Serialize)]
        struct UserOutput {
            id: String,
            name: String,
        }
        #[derive(Serialize)]
        struct LoginSuccessOutput {
            authenticated: bool,
            user: UserOutput,
            workspace: WorkspaceOutput,
            storage: &'static str,
            storage_path: Option<String>,
        }
        let output = LoginSuccessOutput {
            authenticated: true,
            user: UserOutput {
                id: login_result.user_id().to_string(),
                name: login_result.user_name().to_string(),
            },
            workspace: WorkspaceOutput {
                id: login_result.workspace().id().to_string(),
                name: login_result.workspace().name().to_string(),
                url_key: login_result.workspace().url_key().to_string(),
            },
            storage: storage_kind,
            storage_path,
        };
        println!("{}", format_json(&output));
    } else {
        println!(
            "\u{2713} Authenticated as {} (workspace: {})",
            login_result.user_name(),
            login_result.workspace().name()
        );
    }

    Ok(())
}

fn read_api_key(force_json: bool) -> Result<ApiKey, ApplicationError> {
    let is_tty = std::io::stdin().is_terminal();
    if force_json || !is_tty {
        return Err(ApplicationError::Auth(AuthError::ValidationFailed(
            "--api-key is required in non-interactive mode".to_string(),
        )));
    }
    let raw = {
        eprint!("Enter your Linear API key: ");
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
        buf.trim().to_string()
    };

    if raw.is_empty() {
        return Err(ApplicationError::Auth(AuthError::InvalidKey));
    }

    ApiKey::new(raw).map_err(|e| ApplicationError::Auth(AuthError::ValidationFailed(e.to_string())))
}

async fn run_status(force_json: bool) -> Result<(), ApplicationError> {
    use crate::application::use_cases::resolve_auth::resolve_auth;

    let env_key = std::env::var("LINEAR_API_KEY")
        .ok()
        .and_then(|k| ApiKey::new(k).ok());

    let stores: Vec<Box<dyn CredentialStore>> = vec![
        Box::new(KeyringCredentialStore::new()),
        Box::new(FileCredentialStore::new()),
    ];
    let client = Arc::new(LinearGraphqlClient::new());

    match resolve_auth(env_key, stores, client).await {
        Ok(session) => {
            let source_str: &'static str = match session.source() {
                CredentialSource::EnvVar => "env_var",
                CredentialSource::Keychain => "keychain",
                CredentialSource::File(_) => "file",
            };
            let source_path = match session.source() {
                CredentialSource::File(p) => Some(p.display().to_string()),
                _ => None,
            };

            if should_use_json(force_json) {
                #[derive(Serialize)]
                struct StatusOutput {
                    authenticated: bool,
                    workspace: Option<WorkspaceOutput>,
                    source: &'static str,
                    source_path: Option<String>,
                }
                let output = StatusOutput {
                    authenticated: true,
                    workspace: session.workspace().map(|w| WorkspaceOutput {
                        id: w.id().to_string(),
                        name: w.name().to_string(),
                        url_key: w.url_key().to_string(),
                    }),
                    source: source_str,
                    source_path,
                };
                println!("{}", format_json(&output));
            } else {
                let ws_name = session
                    .workspace()
                    .map(|w| w.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("\u{2713} Authenticated (workspace: {ws_name}, source: {source_str})");
            }
        }
        Err(auth_err) => {
            if should_use_json(force_json) {
                let error_code: &'static str = match &auth_err {
                    AuthError::NotAuthenticated => "not_authenticated",
                    AuthError::InvalidKey => "invalid_key",
                    AuthError::NetworkError(_) | AuthError::ValidationFailed(_) => "network_error",
                    _ => "not_authenticated",
                };
                #[derive(Serialize)]
                struct NotAuthOutput {
                    authenticated: bool,
                    error: String,
                    error_code: &'static str,
                }
                let output = NotAuthOutput {
                    authenticated: false,
                    error: auth_err.to_string(),
                    error_code,
                };
                println!("{}", format_json(&output));
            } else {
                eprintln!(
                    "\u{2717} Not authenticated. Run `linear auth login` or set LINEAR_API_KEY."
                );
            }
            return Err(ApplicationError::Auth(auth_err));
        }
    }
    Ok(())
}

async fn run_logout(dry_run: bool, force_json: bool) -> Result<(), ApplicationError> {
    use crate::application::use_cases::logout::LogoutUseCase;

    let stores: Vec<Box<dyn CredentialStore>> = vec![
        Box::new(KeyringCredentialStore::new()),
        Box::new(FileCredentialStore::new()),
    ];
    let use_case = LogoutUseCase::new();
    let result = use_case
        .execute(stores, dry_run)
        .await
        .map_err(ApplicationError::Auth)?;

    if should_use_json(force_json) {
        if dry_run {
            #[derive(Serialize)]
            struct DryRunOutput {
                logged_out: bool,
                would_remove: Vec<StorageEntry>,
                dry_run: bool,
            }
            let output = DryRunOutput {
                logged_out: false,
                would_remove: to_storage_entries(&result),
                dry_run: true,
            };
            println!("{}", format_json(&output));
        } else {
            #[derive(Serialize)]
            struct LogoutOutput {
                logged_out: bool,
                removed: Vec<StorageEntry>,
                dry_run: bool,
            }
            let logged_out = !result.is_empty();
            let output = LogoutOutput {
                logged_out,
                removed: to_storage_entries(&result),
                dry_run: false,
            };
            println!("{}", format_json(&output));
        }
    } else if dry_run {
        if result.is_empty() {
            println!("No credentials found. Nothing to remove.");
        } else {
            println!("Would remove: keychain entry (service: linear-cli, account: default)");
            println!("No changes made (--dry-run).");
        }
    } else if result.is_empty() {
        println!("No credentials found. Nothing to remove.");
    } else {
        println!("\u{2713} Signed out. Credentials removed from keychain.");
    }

    Ok(())
}

#[derive(Serialize)]
struct StorageEntry {
    storage: String,
    path: Option<String>,
}

fn to_storage_entries(kinds: &[StorageKind]) -> Vec<StorageEntry> {
    kinds
        .iter()
        .map(|k| match k {
            StorageKind::Keychain => StorageEntry {
                storage: "keychain".to_string(),
                path: None,
            },
            StorageKind::File(p) => StorageEntry {
                storage: "file".to_string(),
                path: Some(p.display().to_string()),
            },
        })
        .collect()
}
