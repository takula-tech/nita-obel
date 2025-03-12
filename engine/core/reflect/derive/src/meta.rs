use obel_reflect_utils::ObelManifest;
use syn::Path;

/// Returns the correct path for `obel_reflect` crate.
pub(crate) fn get_obel_reflect_path() -> Path {
    ObelManifest::shared().get_path("obel_reflect")
}
