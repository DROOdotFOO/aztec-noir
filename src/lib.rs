use std::fs;
use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, Command, LanguageServerId, Result, Worktree,
};

/// Returns the user-configured nargo path if set, otherwise calls the discovery fallback.
///
/// The user path is returned as-is: we do NOT stat it. Zed extensions run inside a WASI
/// sandbox with preopened directories, so `fs::metadata` on an arbitrary absolute path
/// (e.g. `/opt/noir/bin/nargo`) returns `Err` even when the file exists -- previously
/// that broke every valid `lsp.nargo.binary.path` setting. Let Zed surface its own
/// error if the path can't be launched.
fn resolve_binary_path<F: FnOnce() -> Result<String>>(
    user_path: Option<&str>,
    fallback: F,
) -> Result<String> {
    if let Some(path) = user_path {
        return Ok(path.to_string());
    }
    fallback()
}

struct ZoirExtension {
    cached_binary_path: Option<String>,
}

impl ZoirExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        let (platform, arch) = zed::current_platform();

        // Windows doesn't have pre-built binaries from noir-lang
        if platform == zed::Os::Windows {
            // Still check PATH first in case user built from source
            if let Some(path) = worktree.which("nargo") {
                return Ok(path);
            }
            return Err("Noir does not provide pre-built Windows binaries. \
                Please build nargo from source and add it to your PATH: \
                https://noir-lang.org/docs/getting_started/installation/"
                .to_string());
        }

        // Try PATH first (respects noirup installations)
        if let Some(path) = worktree.which("nargo") {
            return Ok(path);
        }

        // Try cached path
        if let Some(path) = self
            .cached_binary_path
            .as_ref()
            .filter(|p| fs::metadata(p).is_ok())
        {
            return Ok(path.clone());
        }

        // Download from GitHub releases
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "noir-lang/noir",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "nargo-aarch64-apple-darwin.tar.gz",
            (zed::Os::Mac, zed::Architecture::X8664) => "nargo-x86_64-apple-darwin.tar.gz",
            (zed::Os::Linux, zed::Architecture::Aarch64) => {
                "nargo-aarch64-unknown-linux-gnu.tar.gz"
            }
            (zed::Os::Linux, zed::Architecture::X8664) => "nargo-x86_64-unknown-linux-gnu.tar.gz",
            _ => return Err(format!("unsupported platform: {:?} {:?}", platform, arch)),
        };

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no asset found for {}", asset_name))?;

        let version_dir = format!("nargo-{}", release.version);
        let binary_path = format!("{}/nargo", version_dir);

        if fs::metadata(&binary_path).is_err() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download nargo: {}", e))?;

            zed::make_file_executable(&binary_path)?;

            // Cleanup old versions
            fs::read_dir(".")
                .map_err(|e| format!("failed to read directory: {}", e))?
                .flatten()
                .filter(|entry| {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    name_str.starts_with("nargo-") && name_str != version_dir
                })
                .for_each(|entry| {
                    let name_str = entry.file_name().to_string_lossy().into_owned();
                    if let Err(e) = fs::remove_dir_all(entry.path()) {
                        eprintln!("warning: failed to remove old version {}: {}", name_str, e);
                    }
                });
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for ZoirExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let settings = LspSettings::for_worktree("nargo", worktree).ok();
        let binary = settings.as_ref().and_then(|s| s.binary.as_ref());

        let binary_path = resolve_binary_path(binary.and_then(|b| b.path.as_deref()), || {
            self.language_server_binary_path(language_server_id, worktree)
        })?;

        let mut args = vec!["lsp".to_string()];
        if let Some(extra_args) = binary.and_then(|b| b.arguments.as_ref()) {
            args.extend(extra_args.iter().cloned());
        }

        let env = binary
            .and_then(|b| b.env.as_ref())
            .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        Ok(Command {
            command: binary_path,
            args,
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("nargo", worktree)
            .ok()
            .and_then(|s| s.initialization_options);
        Ok(settings)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("nargo", worktree)
            .ok()
            .and_then(|s| s.settings);
        Ok(settings)
    }
}

zed::register_extension!(ZoirExtension);

#[cfg(test)]
mod tests {
    use super::resolve_binary_path;

    #[test]
    fn user_path_takes_precedence_over_fallback() {
        let result = resolve_binary_path(Some("/opt/noir/bin/nargo"), || {
            panic!("fallback must not run when user_path is set")
        });
        assert_eq!(result.as_deref(), Ok("/opt/noir/bin/nargo"));
    }

    #[test]
    fn no_user_path_uses_fallback() {
        let result = resolve_binary_path(None, || Ok("/fallback/nargo".to_string()));
        assert_eq!(result.as_deref(), Ok("/fallback/nargo"));
    }

    /// Regression: previously we did `fs::metadata(path).is_err()` on the user-supplied
    /// path. Inside Zed's WASI sandbox, that always fails for paths outside the
    /// preopened working directory -- which is exactly what users configure (e.g.
    /// `/opt/noir/bin/nargo`). The fix is to pass the path through untouched and let
    /// Zed produce its own launch error. This test pins that contract.
    #[test]
    fn user_path_is_not_stat_checked() {
        let nonexistent = "/definitely/does/not/exist/on/any/system/nargo";
        let result = resolve_binary_path(Some(nonexistent), || {
            panic!("fallback must not run when user_path is set")
        });
        assert_eq!(result.as_deref(), Ok(nonexistent));
    }
}
