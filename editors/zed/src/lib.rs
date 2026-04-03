use std::path::Path;

use zed_extension_api as zed;

struct NeplExtension;

impl zed::Extension for NeplExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != "nepl-lsp" {
            return Err(format!("unsupported language server: {language_server_id}"));
        }

        let worktree_root = worktree.root_path();
        let command = resolve_nepl_lsp_binary(worktree)
            .ok_or_else(|| "failed to find `nepl-lsp`; build it in this repo or place it on PATH".to_string())?;
        let mut env = worktree.shell_env();
        env.push((
            "NEPL_STDLIB_ROOT".to_string(),
            format!("{worktree_root}/stdlib"),
        ));

        Ok(zed::Command {
            command,
            args: vec![],
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        if language_server_id.as_ref() != "nepl-lsp" {
            return Ok(None);
        }
        let worktree_root = worktree.root_path();
        Ok(Some(zed::serde_json::json!({
            "stdlibRoot": format!("{worktree_root}/stdlib")
        })))
    }
}

fn resolve_nepl_lsp_binary(worktree: &zed::Worktree) -> Option<String> {
    if let Some(path) = worktree.which("nepl-lsp") {
        return Some(path);
    }

    let root = worktree.root_path();
    let candidates = [
        format!("{root}/target/debug/nepl-lsp"),
        format!("{root}/target/release/nepl-lsp"),
        format!("{root}/target/debug/nepl-lsp.exe"),
        format!("{root}/target/release/nepl-lsp.exe"),
    ];
    candidates
        .into_iter()
        .find(|candidate| Path::new(candidate).is_file())
}

zed::register_extension!(NeplExtension);
