use crate::cli::Void;
use acorn::prelude::{create_dir_all, write};
use acorn::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER, SKILL_CLIPBOARD_PROMPT, SKILL_FILENAME, SKILL_PATH};
use acorn::util::Label;
use arboard::Clipboard;
use color_eyre::eyre::eyre;
use directories::ProjectDirs;
use rust_embed::Embed;
use tracing::{info, warn};

#[derive(Embed)]
#[folder = "assets/skills/"]
struct SkillAssets;

pub fn run() -> Void {
    let (skill_file, result) = match SkillAssets::get(SKILL_PATH) {
        | Some(asset) => match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            | Some(dirs) => {
                let skill_dir = dirs.cache_dir().join("skills").join("acorn");
                let skill_file = skill_dir.join(SKILL_FILENAME);
                let result = match create_dir_all(&skill_dir) {
                    | Ok(()) => match write(&skill_file, asset.data.as_ref()) {
                        | Ok(()) => Ok(()),
                        | Err(err) => Err(eyre!("Failed to write skill file — {err}")),
                    },
                    | Err(err) => Err(eyre!("Failed to create skill directory — {err}")),
                };
                (Some(skill_file), result)
            }
            | None => (None, Err(eyre!("Failed to resolve project directories"))),
        },
        | None => (None, Err(eyre!("Embedded skill asset not found: {SKILL_PATH}"))),
    };
    if let Some(path) = skill_file {
        println!("{}", path.to_string_lossy());
    }
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(SKILL_CLIPBOARD_PROMPT)) {
        | Ok(()) => info!("{} Agent prompt copied to clipboard", Label::pass()),
        | Err(err) => warn!("{} Clipboard unavailable: {err}", Label::fail()),
    }
    result
}
