//! The source code in this file is Copyright (c) 2017 Gilad Naaman.
//!
//! Copied and modified as allowed per [`MIT license`], also listed below.
//!
//! Changes made:
//! * Now split to a function which returns the base/pattern instead of a glob walker and a function
//!   which creates the a GlobWalkerBuilder which can be further adapted to one's wishes
//! * Added error handling
//!
//! ```text
//! Copyright (c) 2017 Gilad Naaman
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```
//!
//! [`MIT license`]: https://docs.rs/crate/globwalk/0.8.0/source/LICENSE

use anyhow::anyhow;
use globwalk::GlobWalkerBuilder;
use std::path::PathBuf;

/// Create a generic glob builder
pub fn glob_builder_base<PAT: AsRef<str>>(
    pattern: PAT,
    filters: &[PAT],
) -> anyhow::Result<GlobWalkerBuilder> {
    let (base, pat) = glob_base_unrooted(pattern.as_ref())?;

    let base_str: &str = base
        .to_str()
        .ok_or_else(|| anyhow!("Glob base path is not valid"))?;
    let pat_str: &str = pat
        .to_str()
        .ok_or_else(|| anyhow!("Glob pattern is not valid"))?;

    let mut filters = filters
        .iter()
        .map(|f| f.as_ref())
        .map(when_starts_with_dot_path)
        .collect::<Vec<_>>();
    filters.push(when_starts_with_dot_path(pat_str));

    Ok(GlobWalkerBuilder::from_patterns(
        when_starts_with_dot_path(base_str),
        &filters,
    ))
}

// FIXME: fix relative GlobWalkerBuilder with paths starting with "./" or ".\"
fn when_starts_with_dot_path(path: &str) -> &str {
    if path.starts_with("./") || path.starts_with(".\\") {
        &path[2..]
    } else {
        path
    }
}

pub type BasePath = PathBuf;
pub type Pattern = PathBuf;

/// Determine the longest possible base path
///
/// This function contains a partial copy of the [`globwalker::glob`] function
///
/// [`globwalker::glob`]: https://docs.rs/globwalk/0.8.0/src/globwalk/lib.rs.html#430
fn glob_base_unrooted(path_pattern: &str) -> anyhow::Result<(BasePath, Pattern)> {
    let path_buf = path_pattern.parse::<PathBuf>()?;

    Ok(if path_buf.is_absolute() {
        // If the pattern is an absolute path, split it into the longest base and a pattern.
        let mut base = PathBuf::new();
        let mut pattern = PathBuf::new();
        let mut globbing = false;

        // All `to_str().unwrap()` calls should be valid since the input is a string.
        for c in path_buf.components() {
            let os = c.as_os_str().to_str().unwrap();

            for c in &["*", "{", "}"][..] {
                if os.contains(c) {
                    globbing = true;
                    break;
                }
            }

            if globbing {
                pattern.push(c);
            } else {
                base.push(c);
            }
        }

        (base, pattern)
    } else {
        // If the pattern is relative, start searching from the current directory.
        (".".parse().unwrap(), path_buf)
    })
}
