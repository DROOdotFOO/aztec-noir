use std::fs;
use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, Command, LanguageServerId, Result, Worktree,
};

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

        let binary_path = if let Some(path) = binary.and_then(|b| b.path.as_ref()) {
            if fs::metadata(path).is_err() {
                return Err(format!(
                    "nargo binary not found at {path} (configured via lsp.nargo.binary.path)"
                ));
            }
            path.clone()
        } else {
            self.language_server_binary_path(language_server_id, worktree)?
        };

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
