/* /////////////////////////////////////////////////////////////////////////
 * File:    src/lib.rs
 *
 * Purpose: Primary implementation file for libpath.Rust.
 *
 * Created: 16th April 2021
 * Updated: 16th March 2025
 *
 * Home:    http://stlsoft.org/
 *
 * Copyright (c) 2021-2025, Matthew Wilson and Synesis Information Systems
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * - Redistributions of source code must retain the above copyright notice,
 *   this list of conditions and the following disclaimer.
 * - Redistributions in binary form must reproduce the above copyright
 *   notice, this list of conditions and the following disclaimer in the
 *   documentation and/or other materials provided with the distribution.
 * - Neither the name of the copyright holder nor the names of its
 *   contributors may be used to endorse or promote products derived from
 *   this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
 * IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
 * LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 * NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 * ////////////////////////////////////////////////////////////////////// */


pub mod libpath {

    pub mod util {

        pub mod common {
            #![allow(non_camel_case_types)]
            #![allow(non_snake_case)]

            use fastparse::fastparse::types::PositionalSlice as PoSl;

            #[derive(Debug)]
            #[derive(PartialEq, Eq)]
            pub struct ClassificationResult {
                pub Input :                 PoSl,
                pub FullPath :              PoSl, // not used
                pub Prefix :                PoSl,
                pub Location :              PoSl,
                pub Root :                  PoSl,
                pub Directory :             PoSl,
                pub NumDirectoryParts :     usize,
                pub NumDotsDirectoryParts : usize,
                pub Entry :                 PoSl,
                pub Stem :                  PoSl,
                pub Extension :             PoSl,
                pub FirstInvalid :          PoSl,
            }

            impl ClassificationResult {
                pub fn empty() -> Self {
                    Self {
                        Input :                 PoSl::empty(),
                        FullPath :              PoSl::empty(),
                        Prefix :                PoSl::empty(),
                        Location :              PoSl::empty(),
                        Root :                  PoSl::empty(),
                        Directory :             PoSl::empty(),
                        NumDirectoryParts :     0usize,
                        NumDotsDirectoryParts : 0usize,
                        Entry :                 PoSl::empty(),
                        Stem :                  PoSl::empty(),
                        Extension :             PoSl::empty(),
                        FirstInvalid :          PoSl::empty(),
                    }
                }
            }


            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]

                use super::*;
            }
        }

        pub mod unix {

            use super::common::ClassificationResult;
            use fastparse::fastparse::types::PositionalSlice as PoSl;

            pub mod classification_flags {

                /// T.B.C.
                pub const IGNORE_SLASH_RUNS : i32 = 0x00000001;
                /// T.B.C.
                pub const IGNORE_INVALID_CHARS : i32 = 0x00000002;
                /// T.B.C.
                pub const RECOGNISE_TILDE_HOME : i32 = 0x00000004;
            }


            /// Path classification result
            #[derive(Debug)]
            #[derive(PartialEq)]
            pub enum Classification {
                InvalidSlashRuns = -3,
                InvalidChars = -2,
                Invalid = -1,
                Unknown,
                Empty,
                Relative,
                SlashRooted,
                _Reserved1,
                _Reserved2,
                _Reserved3,
                _Reserved4,
                HomeRooted,
            }

            pub fn path_classify(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification,       // classification
                ClassificationResult, // classification_result
            ) {
                if path.is_empty() {
                    return (
                        // argument list:
                        Classification::Empty,
                        ClassificationResult::empty(),
                    );
                }

                let mut cr = ClassificationResult::empty();

                cr.Input = PoSl::new(0, path.len());

                let (cl, root, path_root_stripped) = classify_root_(path, parse_flags);

                cr.Root = PoSl::new(0, root.len());

                // now search within root-stripped path

                let last_slash = find_last_slash_(path_root_stripped.substring_of(path));

                match last_slash {
                    Some(index) => {
                        // if there's a slash, then there is a directory and, potentially, an entry

                        let dir_len = index + 1;

                        cr.Directory = PoSl::new(root.len(), dir_len);

                        let (num_parts, num_dir_parts) = count_parts_(cr.Directory.substring_of(path), parse_flags);
                        cr.NumDirectoryParts = num_parts;
                        cr.NumDotsDirectoryParts = num_dir_parts;

                        cr.Entry = PoSl::new(root.len() + dir_len, path_root_stripped.len() - dir_len);
                    },
                    None => {
                        cr.Directory = PoSl::new(root.len(), 0);

                        // if there's no slash, then the whole (stripped) path is the entry

                        cr.Entry = PoSl::new(root.len(), path_root_stripped.len());
                    },
                }

                if cr.Entry.is_empty() {
                    cr.Stem = cr.Entry;
                    cr.Extension = cr.Entry;
                } else {
                    let last_entry_dot = cr.Entry.substring_of(path).rfind('.');

                    match last_entry_dot {
                        Some(index) => {
                            // handle special dots directories "." and ".."

                            let mut is_dots = false;

                            if !is_dots && 1 == cr.Entry.len() {
                                is_dots = true;
                            }

                            if !is_dots {
                                if 2 == cr.Entry.len() {
                                    if ".." == cr.Entry.substring_of(path) {
                                        is_dots = true;
                                    }
                                }
                            }

                            if is_dots {
                                cr.Stem = cr.Entry;
                                cr.Extension = PoSl::new(cr.Entry.length, 0);
                            } else {
                                cr.Stem = PoSl::new(cr.Entry.offset, index);
                                cr.Extension = PoSl::new(cr.Entry.offset + index, cr.Entry.length - index);
                            }
                        },
                        None => {
                            cr.Stem = cr.Entry;
                            cr.Extension = PoSl::new(cr.Entry.offset + cr.Entry.length, 0);
                        },
                    }
                }

                cr.Location = PoSl::new(0, cr.Entry.offset);

                (cl, cr)
            }

            pub fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification,
                PoSl, // root
                PoSl, // path_root_stripped
            ) {
                debug_assert!(!path.is_empty());

                {
                    let _ = parse_flags;
                }

                let mut ix = -1;
                let mut tilde_0 = false;
                for c in path.chars() {
                    ix += 1;

                    if '~' == c && 0 == ix {
                        tilde_0 = true;

                        continue;
                    }

                    if char_is_path_name_separator_(c) {
                        if 0 == ix {
                            return (
                                // argument list:
                                Classification::SlashRooted,
                                PoSl::empty(),
                                PoSl::new(0, path.len()),
                            );
                        }

                        if 1 == ix {
                            return (
                                // argument list:
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                            );
                        }
                    }

                    break;
                }

                if 0 == ix && tilde_0 {
                    return (
                        // argument list:
                        Classification::HomeRooted,
                        PoSl::new(0, 1),
                        PoSl::new(1, path.len() - 1),
                    );
                }

                (
                    // argument list:
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
                )
            }

            fn char_is_path_name_separator_(c : char) -> bool {
                c == '/'
            }

            fn find_last_slash_(s : &str) -> Option<usize> {
                s.rfind('/')
            }

            fn count_parts_(
                s : &str,
                parse_flags : i32,
            ) -> (
                usize, // number of parts
                usize, // number of dots parts
            ) {
                {
                    let _ = parse_flags;
                }

                // This function counts the number of directory parts and the
                // number of those that are dots directories

                let mut np = 0usize;
                let mut nd = 0usize;

                let mut prev = 'X';

                let mut num_dots = 0;

                for c in s.chars() {
                    if char_is_path_name_separator_(c) {
                        match num_dots {
                            1 | 2 => nd += 1,
                            _ => (),
                        }

                        if char_is_path_name_separator_(prev) {
                        } else {
                            np += 1;
                        }

                        num_dots = 0;
                    } else {
                        if '.' == c {
                            num_dots += 1;
                        } else {
                            num_dots += 100;
                        }
                    }

                    prev = c;
                }

                (np, nd)
            }


            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]

                use super::*;
            }
        }

        pub mod windows {

            use super::common::ClassificationResult;
            use fastparse::fastparse::types::PositionalSlice as PoSl;

            pub mod classification_flags {

                /// T.B.C.
                pub const IGNORE_SLASH_RUNS : i32 = 0x00000001;
                /// T.B.C.
                pub const IGNORE_INVALID_CHARS : i32 = 0x00000002;
                /// T.B.C.
                pub const RECOGNISE_TILDE_HOME : i32 = 0x00000004;
                /// T.B.C.
                pub const IGNORE_INVALID_CHARS_IN_LONG_PATH : i32 = 0x00000002;
            }

            /// Path classification result
            #[derive(Debug)]
            #[derive(PartialEq)]
            pub enum Classification {
                InvalidSlashRuns = -3,
                InvalidChars = -2,
                Invalid = -1,
                Unknown,
                Empty,
                Relative,
                SlashRooted,
                DriveLetterRelative,
                DriveLetterRooted,
                UncIncomplete,
                UncRooted,
                HomeRooted,
            }

            pub fn path_classify(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification,       // classification
                ClassificationResult, // classification_result
            ) {
                if path.is_empty() {
                    return (
                        // argument list:
                        Classification::Empty,
                        ClassificationResult::empty(),
                    );
                }

                let mut cr = ClassificationResult::empty();

                cr.Input = PoSl::new(0, path.len());

                let (cl, root, path_root_stripped) = classify_root_(path, parse_flags);

                cr.Root = PoSl::new(0, root.len());

                // now search within root-stripped path

                let last_slash = find_last_slash_(path_root_stripped.substring_of(path));

                match last_slash {
                    Some(index) => {
                        // if there's a slash, then there is a directory and, potentially, an entry

                        let dir_len = index + 1;

                        cr.Directory = PoSl::new(root.len(), dir_len);

                        let (num_parts, num_dir_parts) = count_parts_(cr.Directory.substring_of(path), parse_flags);
                        cr.NumDirectoryParts = num_parts;
                        cr.NumDotsDirectoryParts = num_dir_parts;

                        cr.Entry = PoSl::new(root.len() + dir_len, path_root_stripped.len() - dir_len);
                    },
                    None => {
                        cr.Directory = PoSl::new(root.len(), 0);

                        // if there's no slash, then the whole (stripped) path is the entry

                        cr.Entry = PoSl::new(root.len(), path_root_stripped.len());
                    },
                }

                if cr.Entry.is_empty() {
                    cr.Stem = cr.Entry;
                    cr.Extension = cr.Entry;
                } else {
                    let last_entry_dot = cr.Entry.substring_of(path).rfind('.');

                    match last_entry_dot {
                        Some(index) => {
                            // handle special dots directories "." and ".."

                            let mut is_dots = false;

                            if !is_dots && 1 == cr.Entry.len() {
                                is_dots = true;
                            }

                            if !is_dots {
                                if 2 == cr.Entry.len() {
                                    if ".." == cr.Entry.substring_of(path) {
                                        is_dots = true;
                                    }
                                }
                            }

                            if is_dots {
                                cr.Stem = cr.Entry;
                                cr.Extension = PoSl::new(cr.Entry.length, 0);
                            } else {
                                cr.Stem = PoSl::new(cr.Entry.offset, index);
                                cr.Extension = PoSl::new(cr.Entry.offset + index, cr.Entry.length - index);
                            }
                        },
                        None => {
                            cr.Stem = cr.Entry;
                            cr.Extension = PoSl::new(cr.Entry.offset + cr.Entry.length, 0);
                        },
                    }
                }

                cr.Location = PoSl::new(0, cr.Entry.offset);

                (cl, cr)
            }

            pub fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification,
                PoSl, // root
                PoSl, // path_root_stripped
            ) {
                debug_assert!(!path.is_empty());

                {
                    let _ = parse_flags;
                }

                let mut ix = -1;

                let mut c0 : char = '\0';
                let mut c1 : char = '\0';
                let mut c2 : char;

                let mut is_drive_2 = false;

                for c in path.chars() {
                    ix += 1;

                    match ix {
                        0 => c0 = c,
                        1 => c1 = c,
                        2 => c2 = c,
                        _ => (),
                    }


                    if ix == 1 {
                        if char_is_drive_letter_(c0) && ':' == c1 {
                            is_drive_2 = true;
                        }

                        if '~' == c0 && char_is_path_name_separator_(c1) {
                            return (
                                // argument list:
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                            );
                        }
                    }

                    if ix == 2 {
                        if is_drive_2 {
                            let classification = if char_is_path_name_separator_(c) {
                                Classification::DriveLetterRooted
                            } else {
                                Classification::DriveLetterRelative
                            };

                            return (
                                // argument list:
                                classification,
                                PoSl::new(0, 2),
                                PoSl::new(2, path.len() - 2),
                            );
                        }
                    }

                    if char_is_path_name_separator_(c) {
                        if 0 == ix {
                            return (
                                // argument list:
                                Classification::SlashRooted,
                                PoSl::empty(),
                                PoSl::new(0, path.len()),
                            );
                        }

                        break;
                    }
                }

                if 0 == ix && '~' == c0 {
                    return (
                        // argument list:
                        Classification::HomeRooted,
                        PoSl::new(0, 1),
                        PoSl::new(1, path.len() - 1),
                    );
                }

                (
                    // argument list:
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
                )
            }

            fn char_is_path_name_separator_(c : char) -> bool {
                match c {
                    '/' => true,
                    '\\' => true,
                    _ => false,
                }
            }

            fn find_last_slash_(s : &str) -> Option<usize> {
                // TODO: consider rfind(&['/', '\\'][..])

                let last_fslash = s.rfind('/');

                match last_fslash {
                    Some(index1) => {
                        let last_rslash = s[index1 + 1..].rfind('\\');

                        match last_rslash {
                            Some(index2) => {
                                if index2 > index1 {
                                    Some(index2)
                                } else {
                                    Some(index1)
                                }
                            },
                            None => last_fslash,
                        }
                    },
                    None => s.rfind('\\'),
                }
            }

            fn count_parts_(
                s : &str,
                parse_flags : i32,
            ) -> (
                usize, // number of parts
                usize, // number of dots parts
            ) {
                {
                    let _ = parse_flags;
                }

                // This function counts the number of directory parts and the
                // number of those that are dots directories

                let mut np = 0usize;
                let mut nd = 0usize;

                let mut prev = 'X';

                let mut num_dots = 0;

                for c in s.chars() {
                    if char_is_path_name_separator_(c) {
                        match num_dots {
                            1 | 2 => nd += 1,
                            _ => (),
                        }

                        if char_is_path_name_separator_(prev) {
                        } else {
                            np += 1;
                        }

                        num_dots = 0;
                    } else {
                        if '.' == c {
                            num_dots += 1;
                        } else {
                            num_dots += 100;
                        }
                    }

                    prev = c;
                }

                (np, nd)
            }

            fn char_is_drive_letter_(c : char) -> bool {
                match c {
                    'A'..='Z' => true,
                    'a'..='z' => true,
                    _ => false,
                }
            }


            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]

                use super::*;
            }
        }
    }


    #[cfg(test)]
    mod tests {
        #![allow(non_snake_case)]

        use super::*;
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::libpath::util::{
        common::ClassificationResult,
        // unix::*,
        // windows::*,
    };

    use fastparse::fastparse::types::PositionalSlice as PoSl;


    #[allow(non_snake_case)]
    mod unix {

        use crate::libpath::util::unix::{
            classification_flags::*,
            path_classify,
            Classification,
        };

        use super::*;


        #[test]
        fn unix_path_classify_empty() {
            let flag_max = 0 | IGNORE_SLASH_RUNS | IGNORE_INVALID_CHARS | RECOGNISE_TILDE_HOME;

            for flags in 0..=flag_max {
                let (cl, cr) = path_classify("", flags);

                assert_eq!(Classification::Empty, cl);

                assert_eq!(ClassificationResult::empty(), cr);
            }
        }

        #[test]
        fn unix_path_classify_entry_only() {
            {
                let path = "name.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 8), cr.Entry);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.Entry.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "name";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.Entry);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.Entry);
                assert_eq!(PoSl::new(0, 0), cr.Stem);
                assert_eq!(PoSl::new(0, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "ab.";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "a..";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "...";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn unix_path_classify_rel_dir_and_name() {
            let path = "dir/name.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 12), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 4), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 4), cr.Directory);
            assert_eq!(1, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(4, 8), cr.Entry);
            assert_eq!(PoSl::new(4, 4), cr.Stem);
            assert_eq!(PoSl::new(8, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("dir/name.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("dir/", cr.Directory.substring_of(path));
            assert_eq!("name.ext", cr.Entry.substring_of(path));
            assert_eq!("name", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_rel_dir_only() {
            {
                let path = "dir/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.Entry);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("dir/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir/", cr.Directory.substring_of(path));
                assert_eq!("", cr.Entry.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "dir1/dir2/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.Entry);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "dir1/../";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.Entry);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "../dir1/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.Entry);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".././";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 5), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 5), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 5), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(2, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(5, 0), cr.Entry);
                assert_eq!(PoSl::new(5, 0), cr.Stem);
                assert_eq!(PoSl::new(5, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn unix_path_classify_dots1_only() {
            let path = ".";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 1), cr.Entry);
            assert_eq!(PoSl::new(0, 1), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn unix_path_classify_dots2_only() {
            let path = "..";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 2), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 2), cr.Entry);
            assert_eq!(PoSl::new(0, 2), cr.Stem);
            assert_eq!(PoSl::new(2, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn unix_path_classify_slashrooted_path() {
            let path = "/dir/sub-dir/file.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::SlashRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 21), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 13), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(13, 8), cr.Entry);
            assert_eq!(PoSl::new(13, 4), cr.Stem);
            assert_eq!(PoSl::new(17, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.Entry.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_home_path() {
            let path = "~/dir/sub-dir/file.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.Entry);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.Entry.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_home_only() {
            let path = "~";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 1), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 0), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(1, 0), cr.Entry);
            assert_eq!(PoSl::new(1, 0), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("", cr.Entry.substring_of(path));
            assert_eq!("", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }
    }


    #[allow(non_snake_case)]
    mod windows {

        use crate::libpath::util::windows::{
            classification_flags::*,
            path_classify,
            Classification,
        };

        use super::*;


        #[test]
        fn windows_path_classify_empty() {
            let flag_max =
                0 | IGNORE_SLASH_RUNS | IGNORE_INVALID_CHARS | RECOGNISE_TILDE_HOME | IGNORE_INVALID_CHARS_IN_LONG_PATH;

            for flags in 0..=flag_max {
                let (cl, cr) = path_classify("", flags);

                assert_eq!(Classification::Empty, cl);

                assert_eq!(ClassificationResult::empty(), cr);
            }
        }

        #[test]
        fn windows_path_classify_entry_only() {
            {
                let path = "name.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 8), cr.Entry);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.Entry.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "name";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.Entry);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.Entry);
                assert_eq!(PoSl::new(0, 0), cr.Stem);
                assert_eq!(PoSl::new(0, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "ab.";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "a..";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "...";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.Entry);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_rel_dir_and_name() {
            {
                let path = "dir/name.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 12), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 8), cr.Entry);
                assert_eq!(PoSl::new(4, 4), cr.Stem);
                assert_eq!(PoSl::new(8, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir\name.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 12), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 8), cr.Entry);
                assert_eq!(PoSl::new(4, 4), cr.Stem);
                assert_eq!(PoSl::new(8, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_rel_dir_only() {
            {
                let path = "dir/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.Entry);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("dir/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir/", cr.Directory.substring_of(path));
                assert_eq!("", cr.Entry.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "dir1/dir2/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.Entry);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "dir1/../";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.Entry);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "../dir1/";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.Entry);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".././";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 5), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 5), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 5), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(2, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(5, 0), cr.Entry);
                assert_eq!(PoSl::new(5, 0), cr.Stem);
                assert_eq!(PoSl::new(5, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir\";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.Entry);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir1\dir2\";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.Entry);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_dots1_only() {
            let path = ".";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 1), cr.Entry);
            assert_eq!(PoSl::new(0, 1), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn windows_path_classify_dots2_only() {
            let path = "..";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 2), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 2), cr.Entry);
            assert_eq!(PoSl::new(0, 2), cr.Stem);
            assert_eq!(PoSl::new(2, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn windows_path_classify_slashrooted_path() {
            {
                let path = "/dir/sub-dir/file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.Entry);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"\dir\sub-dir\file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.Entry);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_driverooted_path() {
            {
                let path = "C:/dir/sub-dir/file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 23), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 15), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.Entry);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"C:\dir\sub-dir\file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 23), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 15), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.Entry);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_driverelative_path() {
            {
                let path = "C:dir/sub-dir/file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::DriveLetterRelative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 22), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 14), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(14, 8), cr.Entry);
                assert_eq!(PoSl::new(14, 4), cr.Stem);
                assert_eq!(PoSl::new(18, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!("dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"C:dir\sub-dir\file.ext";
                let (cl, cr) = path_classify(path, 0);

                assert_eq!(Classification::DriveLetterRelative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 22), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 14), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(14, 8), cr.Entry);
                assert_eq!(PoSl::new(14, 4), cr.Stem);
                assert_eq!(PoSl::new(18, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!(r"dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.Entry.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_home_path() {
            let path = "~/dir/sub-dir/file.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.Entry);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.Entry.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn windows_path_classify_home_only() {
            let path = "~";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 1), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 0), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(1, 0), cr.Entry);
            assert_eq!(PoSl::new(1, 0), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("", cr.Entry.substring_of(path));
            assert_eq!("", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }
    }
}


/* ///////////////////////////// end of file //////////////////////////// */
