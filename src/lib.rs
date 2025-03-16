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
                /// The input string's position.
                pub Input :                 PoSl,
                pub FullPath :              PoSl, // not used
                pub Prefix :                PoSl,
                pub Location :              PoSl,
                /// The root part of the path, such as `"/"` in a UNIX path,
                /// `"C:\"` in a Windows path, or `"\\server\share\"` in a
                /// UNC path.
                pub Root :                  PoSl,
                /// The directory part of the path, such as `"dir/"` in a
                /// UNIX path or `"dir\"` in a Windows path.
                pub Directory :             PoSl,
                /// The number of directory parts in the path, which does
                /// include `Root` and `EntryName`.
                pub NumDirectoryParts :     usize,
                /// The number of directory parts in the path that are dots
                /// directories, i.e. `"."`, `".."`.
                pub NumDotsDirectoryParts : usize,
                /// The "file part", if any, which occurs after the last (if
                /// any) path-name separator.
                pub EntryName :             PoSl,
                /// The entry element's stem.
                pub Stem :                  PoSl,
                /// The entry element's extension.
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
                        EntryName :             PoSl::empty(),
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

                        cr.EntryName = PoSl::new(root.len() + dir_len, path_root_stripped.len() - dir_len);
                    },
                    None => {
                        cr.Directory = PoSl::new(root.len(), 0);

                        // if there's no slash, then the whole (stripped) path is the entry

                        cr.EntryName = PoSl::new(root.len(), path_root_stripped.len());
                    },
                }

                if cr.EntryName.is_empty() {
                    cr.Stem = cr.EntryName;
                    cr.Extension = cr.EntryName;
                } else {
                    let last_entry_dot = cr.EntryName.substring_of(path).rfind('.');

                    match last_entry_dot {
                        Some(index) => {
                            // handle special dots directories "." and ".."

                            let mut is_dots = false;

                            if !is_dots && 1 == cr.EntryName.len() {
                                is_dots = true;
                            }

                            if !is_dots {
                                if 2 == cr.EntryName.len() {
                                    if ".." == cr.EntryName.substring_of(path) {
                                        is_dots = true;
                                    }
                                }
                            }

                            if is_dots {
                                cr.Stem = cr.EntryName;
                                cr.Extension = PoSl::new(cr.EntryName.length, 0);
                            } else {
                                cr.Stem = PoSl::new(cr.EntryName.offset, index);
                                cr.Extension = PoSl::new(cr.EntryName.offset + index, cr.EntryName.length - index);
                            }
                        },
                        None => {
                            cr.Stem = cr.EntryName;
                            cr.Extension = PoSl::new(cr.EntryName.offset + cr.EntryName.length, 0);
                        },
                    }
                }

                cr.Location = PoSl::new(0, cr.EntryName.offset);

                (cl, cr)
            }

            /// Examines the path to the degree necessary to be able to
            /// classify it.
            ///
            /// # Parameters:
            /// - `path` - the given path to be classified;
            /// - `parse_flags` - flags that moderate the classification;
            ///
            /// # Returns:
            /// `(classification : Classification, root : PositionalSlice, path_root_stripped : PositionalSlice)`
            fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification, // classification
                PoSl,           // root
                PoSl,           // path_root_stripped
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
                usize, // number_of_parts
                usize, // number_of_dots_parts
            ) {
                {
                    let _ = parse_flags;
                }

                // This function counts the number of directory parts and
                // the number of those that are dots directories

                let mut number_of_parts = 0usize;
                let mut number_of_dots_parts = 0usize;

                let mut prev = 'X';

                let mut num_dots = 0;

                for c in s.chars() {
                    if char_is_path_name_separator_(c) {
                        match num_dots {
                            1 | 2 => number_of_dots_parts += 1,
                            _ => (),
                        }

                        if char_is_path_name_separator_(prev) {
                        } else {
                            number_of_parts += 1;
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

                (number_of_parts, number_of_dots_parts)
            }


            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]

                use super::*;


                #[test]
                fn char_is_path_name_separator__1() {
                    assert!(char_is_path_name_separator_('/'));
                    assert!(!char_is_path_name_separator_('\\'));

                    assert!(!char_is_path_name_separator_('a'));
                    assert!(!char_is_path_name_separator_(':'));
                    assert!(!char_is_path_name_separator_(';'));
                    assert!(!char_is_path_name_separator_('-'));
                }
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

                        cr.EntryName = PoSl::new(root.len() + dir_len, path_root_stripped.len() - dir_len);
                    },
                    None => {
                        cr.Directory = PoSl::new(root.len(), 0);

                        // if there's no slash, then the whole (stripped) path is the entry

                        cr.EntryName = PoSl::new(root.len(), path_root_stripped.len());
                    },
                }

                if cr.EntryName.is_empty() {
                    cr.Stem = cr.EntryName;
                    cr.Extension = cr.EntryName;
                } else {
                    let last_entry_dot = cr.EntryName.substring_of(path).rfind('.');

                    match last_entry_dot {
                        Some(index) => {
                            // handle special dots directories "." and ".."

                            let mut is_dots = false;

                            if !is_dots && 1 == cr.EntryName.len() {
                                is_dots = true;
                            }

                            if !is_dots {
                                if 2 == cr.EntryName.len() {
                                    if ".." == cr.EntryName.substring_of(path) {
                                        is_dots = true;
                                    }
                                }
                            }

                            if is_dots {
                                cr.Stem = cr.EntryName;
                                cr.Extension = PoSl::new(cr.EntryName.length, 0);
                            } else {
                                cr.Stem = PoSl::new(cr.EntryName.offset, index);
                                cr.Extension = PoSl::new(cr.EntryName.offset + index, cr.EntryName.length - index);
                            }
                        },
                        None => {
                            cr.Stem = cr.EntryName;
                            cr.Extension = PoSl::new(cr.EntryName.offset + cr.EntryName.length, 0);
                        },
                    }
                }

                cr.Location = PoSl::new(0, cr.EntryName.offset);

                (cl, cr)
            }

            /// Examines the path to the degree necessary to be able to
            /// classify it.
            ///
            /// # Parameters:
            /// - `path` - the given path to be classified;
            /// - `parse_flags` - flags that moderate the classification;
            ///
            /// # Returns:
            /// `(classification : Classification, root : PositionalSlice, path_root_stripped : PositionalSlice)`
            fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification, // classification
                PoSl,           // root
                PoSl,           // path_root_stripped
            ) {
                debug_assert!(!path.is_empty());

                {
                    let _ = parse_flags;
                }

                let mut c0 : char = '\0';
                let mut c1 : char = '\0';
                let mut c2 : char;

                let mut is_drive_2 = false;

                let mut ix = -1;

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
                usize, // number_of_parts
                usize, // number_of_dots_parts
            ) {
                {
                    let _ = parse_flags;
                }

                // This function counts the number of directory parts and the
                // number of those that are dots directories

                let mut number_of_parts = 0usize;
                let mut number_of_dots_parts = 0usize;

                let mut prev = 'X';

                let mut num_dots = 0;

                for c in s.chars() {
                    if char_is_path_name_separator_(c) {
                        match num_dots {
                            1 | 2 => number_of_dots_parts += 1,
                            _ => (),
                        }

                        if char_is_path_name_separator_(prev) {
                        } else {
                            number_of_parts += 1;
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

                (number_of_parts, number_of_dots_parts)
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


                #[test]
                fn char_is_drive_letter__1() {
                    assert!(char_is_drive_letter_('a'));
                    assert!(char_is_drive_letter_('A'));
                    assert!(char_is_drive_letter_('c'));
                    assert!(char_is_drive_letter_('C'));
                    assert!(char_is_drive_letter_('z'));
                    assert!(char_is_drive_letter_('Z'));

                    assert!(!char_is_drive_letter_(':'));
                    assert!(!char_is_drive_letter_('/'));
                    assert!(!char_is_drive_letter_('.'));
                }

                #[test]
                fn char_is_path_name_separator__1() {
                    assert!(char_is_path_name_separator_('/'));
                    assert!(char_is_path_name_separator_('\\'));

                    assert!(!char_is_path_name_separator_('a'));
                    assert!(!char_is_path_name_separator_(':'));
                    assert!(!char_is_path_name_separator_(';'));
                    assert!(!char_is_path_name_separator_('-'));
                }

                #[test]
                fn classify_root__1() {
                }

                #[test]
                fn count_parts__1() {
                }

                #[test]
                fn find_last_slash__1() {
                }
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
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 8), cr.EntryName);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.EntryName.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "name";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.EntryName);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("name", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("name", cr.EntryName.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = ".ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.EntryName);
                assert_eq!(PoSl::new(0, 0), cr.Stem);
                assert_eq!(PoSl::new(0, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(".ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!(".ext", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "ab.";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("ab.", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("ab.", cr.EntryName.substring_of(path));
            }

            {
                let path = "a..";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("a..", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("a..", cr.EntryName.substring_of(path));
            }

            {
                let path = "...";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("...", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("...", cr.EntryName.substring_of(path));
            }
        }

        #[test]
        fn unix_path_classify_rel_dir_and_name() {
            let path = "dir/name.ext";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 12), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 4), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 4), cr.Directory);
            assert_eq!(1, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(4, 8), cr.EntryName);
            assert_eq!(PoSl::new(4, 4), cr.Stem);
            assert_eq!(PoSl::new(8, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("dir/name.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("dir/", cr.Directory.substring_of(path));
            assert_eq!("name.ext", cr.EntryName.substring_of(path));
            assert_eq!("name", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_rel_dir_only() {
            {
                let path = "dir/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.EntryName);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("dir/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "dir1/dir2/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.EntryName);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "dir1/../";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.EntryName);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "../dir1/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.EntryName);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".././";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 5), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 5), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 5), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(2, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(5, 0), cr.EntryName);
                assert_eq!(PoSl::new(5, 0), cr.Stem);
                assert_eq!(PoSl::new(5, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "dir-1/../././././././././././abc";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 32), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 29), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 29), cr.Directory);
                assert_eq!(12, cr.NumDirectoryParts);
                assert_eq!(11, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(29, 3), cr.EntryName);
                assert_eq!(PoSl::new(29, 3), cr.Stem);
                assert_eq!(PoSl::new(32, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn unix_path_classify_dots1_only() {
            let path = ".";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 1), cr.EntryName);
            assert_eq!(PoSl::new(0, 1), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn unix_path_classify_dots2_only() {
            let path = "..";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 2), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 2), cr.EntryName);
            assert_eq!(PoSl::new(0, 2), cr.Stem);
            assert_eq!(PoSl::new(2, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn unix_path_classify_dotsnondots1() {
            let path = "...";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 3), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 3), cr.EntryName);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn unix_path_classify_slashrooted_path() {
            let path = "/dir/sub-dir/file.ext";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::SlashRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 21), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 13), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(13, 8), cr.EntryName);
            assert_eq!(PoSl::new(13, 4), cr.Stem);
            assert_eq!(PoSl::new(17, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.EntryName.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_home_path() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.EntryName);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.EntryName.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn unix_path_classify_home_only() {
            let path = "~";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 1), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 0), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(1, 0), cr.EntryName);
            assert_eq!(PoSl::new(1, 0), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("", cr.EntryName.substring_of(path));
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
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 8), cr.EntryName);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.EntryName.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "name";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.EntryName);
                assert_eq!(PoSl::new(0, 4), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 4), cr.EntryName);
                assert_eq!(PoSl::new(0, 0), cr.Stem);
                assert_eq!(PoSl::new(0, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "ab.";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "a..";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "...";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::empty(), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::empty(), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(0, 3), cr.EntryName);
                assert_eq!(PoSl::new(0, 2), cr.Stem);
                assert_eq!(PoSl::new(2, 1), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_rel_dir_and_name() {
            {
                let path = "dir/name.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 12), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 8), cr.EntryName);
                assert_eq!(PoSl::new(4, 4), cr.Stem);
                assert_eq!(PoSl::new(8, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir\name.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 12), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 8), cr.EntryName);
                assert_eq!(PoSl::new(4, 4), cr.Stem);
                assert_eq!(PoSl::new(8, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_rel_dir_only() {
            {
                let path = "dir/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.EntryName);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("dir/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "dir1/dir2/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.EntryName);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "dir1/../";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.EntryName);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = "../dir1/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 8), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 8), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 8), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(1, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(8, 0), cr.EntryName);
                assert_eq!(PoSl::new(8, 0), cr.Stem);
                assert_eq!(PoSl::new(8, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = ".././";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 5), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 5), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 5), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(2, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(5, 0), cr.EntryName);
                assert_eq!(PoSl::new(5, 0), cr.Stem);
                assert_eq!(PoSl::new(5, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir\";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 4), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 4), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 4), cr.Directory);
                assert_eq!(1, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(4, 0), cr.EntryName);
                assert_eq!(PoSl::new(4, 0), cr.Stem);
                assert_eq!(PoSl::new(4, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }

            {
                let path = r"dir1\dir2\";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 10), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 10), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 10), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(10, 0), cr.EntryName);
                assert_eq!(PoSl::new(10, 0), cr.Stem);
                assert_eq!(PoSl::new(10, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());
            }
        }

        #[test]
        fn windows_path_classify_dots1_only() {
            let path = ".";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 1), cr.EntryName);
            assert_eq!(PoSl::new(0, 1), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn windows_path_classify_dots2_only() {
            let path = "..";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 2), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::empty(), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(0, 2), cr.EntryName);
            assert_eq!(PoSl::new(0, 2), cr.Stem);
            assert_eq!(PoSl::new(2, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn windows_path_classify_root() {
            let path = "C:/";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::DriveLetterRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 3), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 3), cr.Location);
            assert_eq!(PoSl::new(0, 2), cr.Root);
            assert_eq!(PoSl::new(2, 1), cr.Directory);
            assert_eq!(1, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(3, 0), cr.EntryName);
            assert_eq!(PoSl::new(3, 0), cr.Stem);
            assert_eq!(PoSl::new(3, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());
        }

        #[test]
        fn windows_path_classify_slashrooted_path() {
            {
                let path = "/dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.EntryName);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"\dir\sub-dir\file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.EntryName);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_driverooted_path() {
            {
                let path = "C:/dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 23), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 15), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.EntryName);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"C:\dir\sub-dir\file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 23), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 15), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 13), cr.Directory);
                assert_eq!(3, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.EntryName);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_driverelative_path() {
            {
                let path = "C:dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRelative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 22), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 14), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(14, 8), cr.EntryName);
                assert_eq!(PoSl::new(14, 4), cr.Stem);
                assert_eq!(PoSl::new(18, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!("dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = r"C:dir\sub-dir\file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRelative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 22), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 14), cr.Location);
                assert_eq!(PoSl::new(0, 2), cr.Root);
                assert_eq!(PoSl::new(2, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(14, 8), cr.EntryName);
                assert_eq!(PoSl::new(14, 4), cr.Stem);
                assert_eq!(PoSl::new(18, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!("C:", cr.Root.substring_of(path));
                assert_eq!(r"dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn windows_path_classify_home_path() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 13), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.EntryName);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.EntryName.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn windows_path_classify_home_only() {
            let path = "~";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::HomeRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 1), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 1), cr.Location);
            assert_eq!(PoSl::new(0, 1), cr.Root);
            assert_eq!(PoSl::new(1, 0), cr.Directory);
            assert_eq!(0, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(1, 0), cr.EntryName);
            assert_eq!(PoSl::new(1, 0), cr.Stem);
            assert_eq!(PoSl::new(1, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~", cr.Location.substring_of(path));
            assert_eq!("~", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("", cr.EntryName.substring_of(path));
            assert_eq!("", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }
    }
}


/* ///////////////////////////// end of file //////////////////////////// */
