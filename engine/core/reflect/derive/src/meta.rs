use obel_reflect_utils::ObelManifest;
use syn::Path;

/// Returns the correct path for `bevy_reflect`.
pub(crate) fn get_bevy_reflect_path() -> Path {
    ObelManifest::shared().get_path("bevy_reflect")
}
