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

mod meta_file;
mod dropwatch;

use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs::*;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::meta_file::*;

#[derive(Default, Debug)]
struct AssetConversion {
    path: String,
    output_path: String,
}

impl PartialEq<AssetConversion> for AssetConversion {
    fn eq(&self, other: &AssetConversion) -> bool {
        return self.path == other.path;
    }
}

fn print_help() {
    println!("Proper usage of prefab_converter.exe is as follows\n");
    println!("./prefab_converter.exe [src assets path] [dst assets path] ...");
    println!("\n... = Any number of valid prefab paths (in the source assets path)!");
    println!("\nExample:");
    println!("\n./prefab_converter.exe \"C:/CustomItemsSDK/Assets\" \"C:/MarrowSDK/Assets\"");
}

fn main() {
    //let dropwatch = dropwatch::Dropwatch::new_begin("PROGRAM");

    // Handle arguments
    let args: Vec<String> = env::args().collect();

    let src_assets;
    let dst_assets;

    if args.len() > 1 {
        // Minimum is 4
        if args.len() < 3 {
            print_help();
            return;
        }

        src_assets = args[1].clone();
        dst_assets = args[2].clone();
    } else {
        print_help();
        return;
    }

    // Before we export, create the temp folder
    let _ = create_dir("ConversionOutput");
    let export_path = "./ConversionOutput".to_string();

    let convert_extensions = {
        let mut vec = Vec::<String>::new();

        let default_extensions = vec![".prefab", ".mat", ".asset", ".unity", ".controller"];

        for ext in default_extensions {
            vec.push(String::from(ext));
        }

        if let Ok(file) = read_to_string("./extensions.txt") {
            // For each line, add it to the extension list
            for line in file.lines() {
                // Is this a comment?
                if line.starts_with("#") {
                    continue;
                }

                // Empty line?
                if line.is_empty() {
                    continue;
                }

                let str = String::from(line);

                if !vec.contains(&str) {
                    vec.push(str)
                }
            }
        }

        vec
    };

    println!("-- [Run Info] --");

    println!("Target Extensions:");
    for ext in &convert_extensions {
        println!("\t{}", ext);
    }

    println!("--============--");

    // ==================
    //  Collection stage
    // ==================

    // We read two projects worth of hash files
    // Any overlap between the two is eliminated (we assume the asset already exists properly)
    let mut missing_metas = Vec::<MetaFile>::new();
    let mut remapped_metas = HashMap::<String, MetaFile>::new();

    println!("-- [Collection Stage] --");
    print!("If this is the first time you've done this since rebooting");
    println!(" you might have to wait a second or two for the OS to cache files and directories!");
    println!("Subsequent runs should be much faster though!");
    println!("--====================--");

    {
        println!("Collecting source meta files...");
        let src_metas = collect_meta_files(&src_assets);

        println!("Collecting destination meta files...");
        let dst_metas = collect_meta_files(&dst_assets);

        //let drop = Dropwatch::new_begin("OVERLAPPING");

        println!("Determining missing meta files...");
        for src_meta in &src_metas {
            //println!("{:?}", src_meta);

            let mut same_found = dst_metas
                .par_iter()
                .any(|dst_meta| dst_meta.guid_hash == src_meta.guid_hash);

            if !same_found {
                // Check if a remap is necessary
                let remap_opt = dst_metas
                    .par_iter()
                    .find_first(|dst_meta| -> bool { src_meta.base_hash == dst_meta.base_hash });

                if let Some(remap) = remap_opt {
                    remapped_metas.insert(src_meta.guid.clone(), remap.clone());
                    same_found = true;
                }
            }

            if !same_found {
                missing_metas.push(src_meta.clone());
            }
        }
    }

    //
    // Conversion stage
    //
    println!("-- [Conversion Stage] --");
    println!("Please be patient, conversion may take a while!");
    println!("--====================--");

    let mut convert_queue: Vec<AssetConversion> = args
        .into_iter()
        .skip(3)
        .map(|str| -> AssetConversion {
            let prefab_dir = PathBuf::from(&str);
            let mut relative_export_path = PathBuf::from(&export_path);

            let sanitized = {
                if let Ok(prefab) = prefab_dir.strip_prefix(&src_assets) {
                    prefab
                } else {
                    prefab_dir.as_path()
                }
            };

            relative_export_path.push(sanitized);
            relative_export_path.pop();

            let mut import = PathBuf::from(&str);

            if !import.starts_with(&src_assets) {
                import = PathBuf::from(&src_assets);
                import.push(&str);
            }

            let mut convert = AssetConversion::default();
            convert.path = import.display().to_string();
            convert.output_path = relative_export_path.display().to_string();

            convert
        })
        .collect();

    while let Some(convert) = convert_queue.pop() {
        let prefab_path = Path::new(&convert.path);

        if prefab_path.is_dir() {
            continue;
        }

        let mut prefab_file = File::open(prefab_path).unwrap();

        // Copy over the meta file first (if it doesn't exist)
        let mut meta_path = prefab_path.display().to_string();
        meta_path.push_str(".meta");

        let mut contents = String::new();
        let _size = prefab_file.read_to_string(&mut contents);

        let mut converted_contents = contents.clone();

        // Find all occurrences of "guid"
        for indice in contents.match_indices("guid: ") {
            let guid: String = contents.chars().skip(indice.0 + 6).take(32).collect();

            // Check if this has been remapped
            if let Some(meta_file) = remapped_metas.get(&guid) {
                converted_contents.replace_range(indice.0 + 6..indice.0 + 6 + 32, &meta_file.guid);
                continue;
            }

            // Check if this is in our list of missing ones
            // If so copy it
            let mut need_delete = false;
            let mut delete = 0usize;

            for missing_meta in &missing_metas {
                if missing_meta.guid == guid {
                    // Copy the asset (with and without the meta over)
                    // If the file doesn't exist already, copy it
                    let prefab_dir = PathBuf::from(&missing_meta.directory);
                    let mut relative_export_path = PathBuf::from(&export_path);
                    relative_export_path.push(prefab_dir.strip_prefix(&src_assets).unwrap());

                    let export_path = relative_export_path.display().to_string();
                    let _ = create_dir_all(&export_path);

                    let (asset_src_path, meta_src_path) = missing_meta.get_paths();
                    let (asset_dst_path, meta_dst_path) = missing_meta.get_paths_stem(&export_path);

                    if Path::new(&asset_src_path).exists() && !Path::new(&asset_dst_path).exists() {
                        copy(&asset_src_path, &asset_dst_path).unwrap();
                    }

                    if Path::new(&meta_src_path).exists() && !Path::new(&meta_dst_path).exists() {
                        copy(&meta_src_path, &meta_dst_path).unwrap();
                    }

                    // If this is a prefab, push it to the list of queued conversions
                    // If it hasn't been pushed already!
                    for ext in &convert_extensions {
                        if missing_meta.base_name.ends_with(ext.as_str()) {
                            let mut convert = AssetConversion::default();
                            convert.path = asset_src_path.clone();

                            if !convert_queue.contains(&convert) {
                                println!("[Conversion]: Queuing referenced asset {:?}", asset_src_path);

                                convert.output_path = relative_export_path.display().to_string();
                                convert_queue.push(convert);
                            }

                            break;
                        }
                    }

                    // After being successfully copied, this is removed from the missing list
                    // This prevents prefab duplication / overwriting
                    need_delete = true;
                    break;

                    //println!("MATCH: {} = {:?}", guid, missing_meta)
                }

                delete += 1;
            }

            if need_delete {
                missing_metas.remove(delete);
            }
        }

        let _ = create_dir_all(&convert.output_path);

        let mut file_path = PathBuf::from(convert.output_path);
        file_path.push(prefab_path.file_name().unwrap());

        write(&file_path, converted_contents).unwrap();

        let mut extension = file_path.extension().unwrap().to_str().unwrap().to_string();
        extension.push_str(".meta");

        file_path.set_extension(extension);
        let _ = copy(meta_path, &file_path);
    }
}
