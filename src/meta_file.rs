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

pub mod collector;
pub mod meta_file;

pub use collector::*;
pub use meta_file::*;

use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub fn collect_recurse<P: AsRef<Path>>(path: P, dirs: &mut Vec<PathBuf>) {
    for entry_result in read_dir(path).expect("Failed to read given path!") {
        // If we can't read a meta file we probably shouldn't be in here
        let entry = entry_result.expect("Failed to read file in given path!");

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                dirs.push(entry.path());
                collect_recurse(entry.path(), dirs);
            }
        }
    }
}

pub fn collect_meta_files(path: &String) -> Vec<MetaFile> {
    // First fetch all the directories within a project
    let mut dirs = Vec::<PathBuf>::new();
    collect_recurse(path, &mut dirs);

    // Then collect them
    //println!("Collecting meta files...");
    let collect_multi = true;

    return if collect_multi {
        //let drop = dropwatch::Dropwatch::new_begin("META_COLLECT");

        let collector = MetaFileCollector::new(dirs);
        collector.wait();

        collector.consume()
    } else {
        //let drop = dropwatch::Dropwatch::new_begin("META_COLLECT");

        let mut metas = Vec::<MetaFile>::new();
        for path in dirs {
            for entry_result in read_dir(path).expect("Failed to read given path!") {
                // If we can't read a meta file we probably shouldn't be in here
                let entry = entry_result.expect("Failed to read file in given path!");

                if let Some(extension) = entry.path().extension() {
                    if extension == "meta" {
                        let meta = MetaFile::read_from_path(&entry.path()).unwrap();

                        //println!("{:?}", meta);
                        metas.push(meta);
                    }
                }
            }
        }

        metas
    };
}
