pub mod seven_z;
pub mod tar_gz;
pub mod tar_shared;
pub mod tar_xz;
pub mod tar_zst;
pub mod zip;

use anyhow::Result;
use std::path::Path;

/// Defines the behavior that the app expects
/// from an archive builder.
pub trait AppArchive {
    /// Adds a file to the current builder.
    /// # Arguments
    /// - full_path - the full path to the file being added.
    /// - archive_path - the relative path within the archive.
    fn add_file_to_archive(&mut self, full_path: &Path, archive_path: &Path) -> Result<()>;

    /// Finishes the current builder and writes the
    /// result to disk under the given save_filename.
    fn save_archive(&mut self, save_filename: &str) -> Result<()>;

    /// Returns a message with the proper extension
    /// for use with info messages.
    fn save_msg(&self, name: &str) -> String;

    /// Unpacks an archive file to the given destination.
    fn extract_archive_file(&self, archive_path: &Path, destination: &Path) -> Result<()>;

    /// Takes the current builder and unpacks its contents.
    fn extract_current(&mut self, destination: &Path) -> Result<()>;

    /// Removes a file from the current archive builder.
    /// With the way that these archive builders are set up,
    /// there is no way to remove a file and leave the rest
    /// of the builder intact. This means that the entire archive
    /// must be rebuilt with all of the files except the file
    /// targeted for removal.
    fn remove_from_archive(&mut self, file_to_remove: &Path) -> Result<()>;
}
