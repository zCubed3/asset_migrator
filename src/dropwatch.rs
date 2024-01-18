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

#![allow(dead_code)]

use std::time;

/// Measures time from the creation of this watch to when it exists scope
pub struct Dropwatch {
    start: Option<time::Instant>,
    id: String,
}

impl Dropwatch {
    pub fn new(id: &str) -> Self {
        Dropwatch {
            start: None,
            id: id.to_string(),
        }
    }

    pub fn begin(&mut self) {
        self.start = Some(time::Instant::now());
    }

    pub fn new_begin(id: &str) -> Self {
        let mut s = Self::new(id);
        s.begin();
        s
    }
}

impl Drop for Dropwatch {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            println!(
                "{}: {} seconds elapsed",
                self.id,
                (time::Instant::now() - start).as_secs_f32()
            );
        }
    }
}
