use acorn::io::ApiResult;
use color_eyre::eyre::eyre;
use rust_embed::Embed;

pub(crate) const DRAFT_PLACEHOLDER_ASSET: &str = "draft-placeholder-1920x1280.png";
#[derive(Embed)]
#[folder = "assets/images/"]
pub(crate) struct Image;
pub(crate) fn draft_placeholder_bytes() -> ApiResult<Vec<u8>> {
    Image::get(DRAFT_PLACEHOLDER_ASSET)
        .map(|file| file.data.into_owned())
        .ok_or_else(|| eyre!("Missing embedded image asset: {DRAFT_PLACEHOLDER_ASSET}"))
}
