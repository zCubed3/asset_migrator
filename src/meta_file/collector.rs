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

use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{spawn, JoinHandle};

use super::meta_file::MetaFile;

/// Spawns threads and collects meta files from an internal worklist
pub struct MetaFileCollector {
    threads: Vec<JoinHandle<()>>,
    meta_files: Arc<Mutex<Vec<MetaFile>>>,
    condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl MetaFileCollector {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let mut threads = Vec::<JoinHandle<()>>::new();
        let work_paths = Arc::new(Mutex::new(paths));
        let meta_files = Arc::new(Mutex::new(Vec::<MetaFile>::new()));
        let condvar = Arc::new((Mutex::new(false), Condvar::new()));

        // TODO: Get hardware concurrency?
        let thread_count = std::thread::available_parallelism().unwrap().get();

        #[cfg(debug_assertions)]
        {
            println!("DEBUG: COLLECTING WITH {} THREADS", thread_count);
        }

        for _ in 0usize..thread_count {
            let work_paths = Arc::clone(&work_paths);
            let meta_files = Arc::clone(&meta_files);
            let condvar = Arc::clone(&condvar);

            threads.push(spawn(move || {
                MetaFileCollector::collector_loop(condvar, work_paths, meta_files)
            }));
        }

        Self {
            threads,
            meta_files,
            condvar,
        }
    }

    pub fn wait(&self) {
        {
            let (lock, cvar) = &*self.condvar;

            let mut notified = lock.lock().unwrap();
            notified = cvar.wait(notified).unwrap();
        }
    }

    pub fn consume(self) -> Vec<MetaFile> {
        // Ensure all the threads have exited first
        loop {
            let mut all_finished = true;
            for thread in &self.threads {
                if !thread.is_finished() {
                    all_finished = false;
                    break;
                }
            }

            if all_finished {
                break;
            }
        }

        return Arc::try_unwrap(self.meta_files)
            .unwrap()
            .into_inner()
            .unwrap();
    }

    fn collector_loop(
        condvar: Arc<(Mutex<bool>, Condvar)>,
        work_paths: Arc<Mutex<Vec<PathBuf>>>,
        meta_files: Arc<Mutex<Vec<MetaFile>>>,
    ) {
        loop {
            let path: Option<PathBuf>;
            let notify: bool;

            {
                let mut lock = work_paths.lock().unwrap();
                path = lock.pop();
                notify = lock.is_empty();
            }

            if let Some(path) = path {
                // Read the files first
                let mut metas = Vec::<MetaFile>::new();

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

                {
                    let mut lock = meta_files.lock().unwrap();
                    lock.append(&mut metas);
                }

                if notify {
                    let (lock, cvar) = &*condvar;

                    let mut notified = lock.lock().unwrap();
                    *notified = true;

                    cvar.notify_one();
                }
            } else {
                break;
            }
        }
    }
}
