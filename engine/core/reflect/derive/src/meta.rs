use obel_reflect_utils::ObelManifest;
use syn::Path;

/// Returns the correct path for `obel_reflect` crate.
pub(crate) fn get_path_to_obel_reflect() -> Path {
    ObelManifest::shared().get_path("obel_reflect")
}
