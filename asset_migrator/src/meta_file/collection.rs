// ===================================================================================
//  BSD 3-Clause License
//
//  Copyright (c) 2023-2024, Liam Reese (zCubed3)
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

use crate::meta_file::MetaFile;

use rayon::prelude::*;
use std::fs::{read_dir, DirEntry};
use std::path::{Path, PathBuf};

// ======================
//  Meta File Collection
// ======================
fn collect_recurse<P: AsRef<Path>>(path: P, dirs: &mut Vec<PathBuf>) {
    // https://stackoverflow.com/questions/77608489/rust-rayon-collect-options-into-vector
    let entries: Vec<DirEntry> = read_dir(path)
        .expect("Failed to read directory!")
        .filter_map(|e| e.ok())
        .collect();

    let mut found_dirs = entries
        .into_par_iter()
        .map(|dir| -> Vec<PathBuf> {
            if !dir.file_type().unwrap().is_dir() {
                return vec![];
            }

            let mut paths = Vec::<PathBuf>::new();
            paths.push(dir.path());

            collect_recurse(dir.path(), &mut paths);

            paths
        })
        .flatten_iter()
        .collect();

    dirs.append(&mut found_dirs);
}

pub fn collect_meta_files(path: &String) -> Vec<MetaFile> {
    let mut dirs = vec![];
    collect_recurse(path, &mut dirs);

    dirs.into_par_iter()
        .map(|dir| -> Vec<MetaFile> {
            let read = read_dir(dir).expect("Failed to read directory!");

            read.into_iter()
                .filter_map(|e| e.ok())
                .filter_map(|entry| -> Option<MetaFile> {
                    let path = entry.path();

                    if let Some(ext) = path.extension() {
                        if ext == "meta" {
                            return MetaFile::read_from_path(&path);
                        }
                    }

                    None
                })
                .collect()
        })
        .flatten()
        .collect()
}
