// ===================================================================================
//  BSD 3-Clause License
//
//  Copyright (c) 2023-2024, Liam R. (zCubed3)
//
//  Redistribution and use in source and binary forms, with or without
//  modification, are permitted provided that the following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this
//     list of conditions and the following disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice,
//     this list of conditions and the following disclaimer in the documentation
//     and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its
//     contributors may be used to endorse or promote products derived from
//     this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
//  AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//  IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
//  FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//  DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
//  CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//  OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
//  OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// ===================================================================================

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Unity meta file (GUID only)
#[derive(Debug, Default, Clone)]
pub struct MetaFile {
    /// The directory of this meta file
    pub directory: String,

    /// The base name of this meta file
    pub base_name: String,

    /// Unity's GUID for this asset
    pub guid: String,

    /// Hash of the GUID (for faster checking)
    pub guid_hash: u64,

    /// Hash of the base name
    pub base_hash: u64,
}

impl MetaFile {
    /// Reads a meta file, grabs the GUID and returns it
    pub fn read_from_path(path: &PathBuf) -> Option<Self> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);

            let mut meta_file = Self::default();

            meta_file.base_name = path
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            meta_file.directory = path.parent().unwrap().display().to_string();

            {
                let mut hasher = DefaultHasher::new();
                meta_file.base_name.hash(&mut hasher);
                meta_file.base_hash = hasher.finish();
            }

            for line in reader.lines() {
                if let Ok(contents) = line {
                    if contents.contains("guid: ") {
                        meta_file.guid = contents.replace("guid:", "").trim().to_string();

                        // Hashing the GUID makes overlap comparison BLAZING FAST :P
                        let mut hasher = DefaultHasher::new();
                        meta_file.guid.hash(&mut hasher);
                        meta_file.guid_hash = hasher.finish();

                        break;
                    }
                }
            }

            if !meta_file.guid.is_empty() {
                return Some(meta_file);
            }
        }

        return None;
    }

    /// Returns the asset and meta file paths with a new stem
    pub fn get_paths_stem<P: AsRef<Path>>(&self, stem: P) -> (String, String) {
        let mut asset_path = PathBuf::new();
        asset_path.push(stem);
        asset_path.push(&self.base_name);

        let asset_path_string = asset_path.display().to_string();

        let mut meta_path_string = asset_path_string.clone();
        meta_path_string.push_str(".meta");

        return (asset_path_string, meta_path_string);
    }

    /// Returns the asset and meta file paths
    pub fn get_paths(&self) -> (String, String) {
        return self.get_paths_stem(&self.directory);
    }
}
