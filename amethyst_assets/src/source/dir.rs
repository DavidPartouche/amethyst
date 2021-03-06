use source::Source;
#[cfg(not(target_os = "android"))]
use std::fs::File;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "android"))]
use std::time::UNIX_EPOCH;
#[cfg(not(target_os = "android"))]
use ResultExt;
use {ErrorKind, Result};

#[cfg(target_os = "android")]
use android_glue::{load_asset, AssetError};
#[cfg(target_os = "android")]
use error::Error;

/// Directory source.
///
/// Please note that there is a default directory source
/// inside the `Loader`, which is automatically used when you call
/// `load`. In case you want another, second, directory for assets,
/// you can instantiate one yourself, too. Please use `Loader::load_from` then.
#[derive(Debug)]
pub struct Directory {
    loc: PathBuf,
}

impl Directory {
    /// Creates a new directory storage.
    pub fn new<P>(loc: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Directory { loc: loc.into() }
    }

    fn path(&self, s_path: &str) -> PathBuf {
        let mut path = self.loc.clone();
        path.extend(Path::new(s_path).iter());

        path
    }
}

impl Source for Directory {
    #[cfg(not(target_os = "android"))]
    fn modified(&self, path: &str) -> Result<u64> {
        #[cfg(feature = "profiler")]
        profile_scope!("dir_modified_asset");
        use std::fs::metadata;

        let path = self.path(path);

        Ok(metadata(&path)
            .chain_err(|| format!("Failed to fetch metadata for {:?}", path))?
            .modified()
            .chain_err(|| "Could not get modification time")?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs())
    }

    #[cfg(target_os = "android")]
    // TODO: Quick hack to make the image draw to Android, need to be properly implemented
    fn modified(&self, _path: &str) -> Result<u64> {
        Ok(0)
    }

    #[cfg(not(target_os = "android"))]
    fn load(&self, path: &str) -> Result<Vec<u8>> {
        #[cfg(feature = "profiler")]
        profile_scope!("dir_load_asset");
        use std::io::Read;

        let path = self.path(path);

        let mut v = Vec::new();
        let mut file = File::open(&path)
            .chain_err(|| format!("Failed to open file {:?}", path))
            .chain_err(|| ErrorKind::Source)?;
        file.read_to_end(&mut v)
            .chain_err(|| format!("Failed to read file {:?}", path))
            .chain_err(|| ErrorKind::Source)?;

        Ok(v)
    }

    #[cfg(target_os = "android")]
    fn load(&self, path: &str) -> Result<Vec<u8>> {
        #[cfg(feature = "profiler")]
        profile_scope!("dir_load_asset");

        load_asset(path).map_err(|e| match e {
            AssetError::AssetMissing => {
                Error::from_kind(ErrorKind::Msg(format!("Failed to open file {}", path)))
            }
            AssetError::EmptyBuffer => {
                Error::from_kind(ErrorKind::Msg(format!("Failed to read file {}", path)))
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::Directory;
    use source::Source;
    use std::path::Path;

    #[test]
    fn loads_asset_from_assets_directory() {
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            "data".as_bytes().to_vec(),
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }

    #[cfg(windows)]
    #[test]
    fn tolerates_backslashed_location_with_forward_slashed_asset_paths() {
        // Canonicalized path on Windows uses backslashes
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/assets")
            .canonicalize()
            .expect("Failed to canonicalize tests/assets directory");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            "data".as_bytes().to_vec(),
            // Use forward slash to declare path
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }
}
