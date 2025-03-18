/* /////////////////////////////////////////////////////////////////////////
 * File:    src/lib.rs
 *
 * Purpose: Primary implementation file for libpath.Rust.
 *
 * Created: 16th April 2021
 * Updated: 17th March 2025
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


            /// Describes the classification.
            ///
            /// A given full path will have the following elements:
            /// - FullPath - the full
            /// - Prefix
            #[derive(Debug)]
            #[derive(PartialEq, Eq)]
            pub struct ClassificationResult {
                /// The input string's position.
                pub Input :                 PoSl,
                /// The full path.
                ///
                /// NOTE: this is not used currently.
                pub FullPath :              PoSl,
                /// The prefix.
                pub Prefix :                PoSl,
                /// TODO
                ///
                /// # Note:
                /// Equivalent to **recls**' `DirectoryPath`.
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
                /// 0-based index of the first invalid character in `Input`.
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

                /*
                use super::*;
                 */
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
                /// T.B.C.
                pub const ASSUME_DIRECTORY : i32 = 0x00000008;
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

                let (cl, root, path_root_stripped, _first_bad_char_index) = classify_root_(path, parse_flags);

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
                        Some(index) if index + 1 < cr.EntryName.len() => {
                            cr.Stem = PoSl::new(cr.EntryName.offset, index);
                            cr.Extension = PoSl::new(cr.EntryName.offset + index, cr.EntryName.len() - index);
                        },
                        _ => {
                            cr.Stem = cr.EntryName;
                            cr.Extension = PoSl::new(cr.EntryName.offset + cr.EntryName.len(), 0);
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
            /// `(classification : Classification, root : PositionalSlice, path_root_stripped : PositionalSlice, first_bad_char_index : Option<usize>)`
            fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification, // classification
                PoSl,           // root
                PoSl,           // path_root_stripped
                Option<usize>,  // first_bad_char_index
            ) {
                debug_assert!(!path.is_empty());

                {
                    let _ = parse_flags;
                }

                let mut tilde_0 = false;

                let mut ix = -1;
                for c in path.chars() {
                    ix += 1;

                    if char_is_invalid_in_path_(c) {
                        return (
                            // argument list:
                            Classification::InvalidChars,
                            PoSl::empty(),
                            PoSl::empty(),
                            Some(ix as usize),
                        );
                    }

                    if '~' == c && 0 == ix {
                        if classification_flags::RECOGNISE_TILDE_HOME
                            == (classification_flags::RECOGNISE_TILDE_HOME & parse_flags)
                        {
                            tilde_0 = true;

                            continue;
                        }
                    }

                    if char_is_path_name_separator_(c) {
                        if 0 == ix {
                            return (
                                // argument list:
                                Classification::SlashRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                                None,
                            );
                        }

                        if 1 == ix && tilde_0 {
                            return (
                                // argument list:
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                                None,
                            );
                        }
                    } else {
                        if 0 == ix {
                            return (
                                // argument list:
                                Classification::Relative,
                                PoSl::empty(),
                                PoSl::new(0, path.len()),
                                None,
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
                        None,
                    );
                }

                (
                    // argument list:
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
                    None,
                )
            }

            /// Evaluates whether a character is invalid in a path entry
            /// name.
            /*
            fn char_is_invalid_in_entryname_(c : char) -> bool {
                match c {
                    // On all Unix, by definition, the character '/' is invalid.
                    '/' => true,
                    // On macOS (through observation), the additional character
                    // values in the range 128-255 is invalid.
                    '\u{0080}'..='\u{00ff}' => true,

                    _ => false,
                }
            }
             */

            /// Evaluates whether a character is invalid in a path.
            fn char_is_invalid_in_path_(c : char) -> bool {
                match c {
                    // On macOS (through observation), the additional character
                    // values in the range 128-255 is invalid.
                    '\u{0080}'..='\u{00ff}' => true,
                    '*' => true,
                    '<' => true,
                    '>' => true,
                    '?' => true,
                    '|' => true,

                    _ => false,
                }
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

                use fastparse::fastparse::types::PositionalSlice as PoSl;


                #[test]
                fn TEST_char_is_invalid_in_path__1() {
                    assert!(char_is_invalid_in_path_('<'));
                    assert!(char_is_invalid_in_path_('>'));
                    assert!(char_is_invalid_in_path_('|'));

                    assert!(!char_is_invalid_in_path_('-'));
                    assert!(!char_is_invalid_in_path_('/'));
                    assert!(!char_is_invalid_in_path_(':'));
                    assert!(!char_is_invalid_in_path_(';'));
                    assert!(!char_is_invalid_in_path_('\\'));
                    assert!(!char_is_invalid_in_path_('a'));
                }

                #[test]
                fn TEST_char_is_path_name_separator__1() {
                    assert!(char_is_path_name_separator_('/'));
                    assert!(!char_is_path_name_separator_('\\'));

                    assert!(!char_is_path_name_separator_('-'));
                    assert!(!char_is_path_name_separator_(':'));
                    assert!(!char_is_path_name_separator_(';'));
                    assert!(!char_is_path_name_separator_('a'));
                }

                #[test]
                fn TEST_classify_root__1() {
                    let test_criteria = &[
                        ("dir", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),
                        ("file.ext", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 8), None),
                        ("/", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),
                        ("/dir", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 3), None),
                        ("/file.ext", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 8), None),
                        ("/dir/", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 4), None),
                        ("/dir/file.ext", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 12), None),
                        ("~", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 1), None),
                        ("~", classification_flags::RECOGNISE_TILDE_HOME, Classification::HomeRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),
                        ("~/", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~/", classification_flags::RECOGNISE_TILDE_HOME, Classification::HomeRooted, PoSl::new(0, 1), PoSl::new(1, 1), None),
                        ("~a", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~a", classification_flags::RECOGNISE_TILDE_HOME, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~a/", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),
                        ("~a/", classification_flags::RECOGNISE_TILDE_HOME, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),

                        ("|a", 0, Classification::InvalidChars, PoSl::empty(), PoSl::empty(), Some(0)),
                        ("a|", 0, Classification::Relative, PoSl::empty(), PoSl::new(0, 2), None),
                    ];

                    for (
                        path,
                        parse_flags,
                        expected_classification,
                        expected_root,
                        expected_path_root_stripped,
                        expected_first_bad_char_index,
                    ) in test_criteria
                    {
                        let (classification, root, path_root_stripped, first_bad_char_index) =
                            classify_root_(*path, *parse_flags);

                        assert_eq!(*expected_classification, classification, "wrong classification {classification:?} ({expected_classification:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_root, root, "wrong root {root:?} ({expected_root:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_path_root_stripped, path_root_stripped, "wrong path-root-stripped {path_root_stripped:?} ({expected_path_root_stripped:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_first_bad_char_index, first_bad_char_index, "wrong first-bad-char-index {first_bad_char_index:?} ({expected_first_bad_char_index:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                    }
                }

                #[test]
                fn TEST_count_parts__1() {
                    let test_criteria = &[
                        ("", 0, 0, 0),
                        ("a", 0, 0, 0),
                        ("a/", 0, 1, 0),
                        ("abc", 0, 0, 0),
                        ("abc/", 0, 1, 0),
                        ("abc/def", 0, 1, 0),
                        ("abc/def/", 0, 2, 0),
                        ("🐻", 0, 0, 0),
                        ("🐻/", 0, 1, 0),
                        ("🐻/🐻‍❄️", 0, 1, 0),
                        ("🐻/🐻‍❄️/", 0, 2, 0),
                        ("/", 0, 1, 0),
                        ("/abc", 0, 1, 0),
                        ("/abc/", 0, 2, 0),
                        ("/abc/def", 0, 2, 0),
                        ("/abc/def/", 0, 3, 0),
                        ("..", 0, 0, 0),
                        ("../", 0, 1, 1),
                        ("/..", 0, 1, 0),
                        ("/../", 0, 2, 1),
                    ];

                    for (s, parse_flags, expected_number_of_parts, expected_number_of_dots_parts) in test_criteria {
                        let (number_of_parts, number_of_dots_parts) = count_parts_(*s, *parse_flags);

                        assert_eq!(*expected_number_of_parts, number_of_parts, "wrong number of parts {number_of_parts} ({expected_number_of_parts} expected) when parsing '{s}'");
                        assert_eq!(*expected_number_of_dots_parts, number_of_dots_parts, "wrong number of dots parts {number_of_dots_parts} ({expected_number_of_dots_parts} expected) when parsing '{s}'");
                    }
                }

                #[test]
                fn TEST_find_last_slash__1() {
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

                let (cl, root, path_root_stripped, _first_bad_char_index) = classify_root_(path, parse_flags);

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
                        Some(index) if index + 1 < cr.EntryName.len() => {
                            cr.Stem = PoSl::new(cr.EntryName.offset, index);
                            cr.Extension = PoSl::new(cr.EntryName.offset + index, cr.EntryName.len() - index);
                        },
                        _ => {
                            cr.Stem = cr.EntryName;
                            cr.Extension = PoSl::new(cr.EntryName.offset + cr.EntryName.len(), 0);
                        },
                    }
                }

                cr.Location = PoSl::new(0, cr.EntryName.offset);

                (cl, cr)
            }

            // Splits the given path into slices.
            //
            // # Note:
            // The splitting is done by byte not by character, because all
            // significant characters are ASCII.
            fn unc_split_<'a>(
                path : &'a str,
                parse_flags : i32,
            ) -> Vec<&'a str>
            {
                let mut v = Vec::with_capacity(1 + path.len() / 10);

                let path_bytes = path.as_bytes();

                let mut prev = b'|'; // anything other than a slash
                let mut from = 0;
                for ix in 0..path.len() {
                    let c = path_bytes[ix];

                    match c {
                        b'/' | b'\\' => {

                            if c == prev {
                                // continue the slash run

                            } else {
                                // push non-slash run slice

                                if from != ix {

                                    let slice0 = &path[from..ix];

                                    // check for drive-relative strings (e.g. "C:dir") and split them
                                    if slice0.len() > 2 && str_begins_with_drive_spec_(slice0) {

                                        let slice1 = &slice0[0..2];
                                        let slice2 = &slice0[2..];

                                        v.push(slice1);
                                        v.push(slice2);
                                    } else {

                                        v.push(slice0);
                                    }

                                    from = ix;
                                }
                            }
                        },
                        _ => {

                            match prev {
                                b'/' | b'\\' => {
                                    // push slash run slice

                                    v.push(&path[from..ix]);
                                    from = ix;
                                },
                                _ => {
                                    // continue the non-slash run

                                },
                            }
                        }
                    }

                    prev = c;
                }

                if from != path.len() {

                    v.push(&path[from..]);
                }

                v
            }

            /// Examines the path to the degree necessary to be able to
            /// classify it.
            ///
            /// # Parameters:
            /// - `path` - the given path to be classified;
            /// - `parse_flags` - flags that moderate the classification;
            ///
            /// # Returns:
            /// `(classification : Classification, root : PositionalSlice, path_root_stripped : PositionalSlice, first_bad_char_index : Option<usize>)`
            fn classify_root_(
                path : &str,
                parse_flags : i32,
            ) -> (
                Classification, // classification
                PoSl,           // root
                PoSl,           // path_root_stripped
                Option<usize>,  // first_bad_char_index
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

                    if char_is_invalid_in_path_(c) {
                        return (
                            // argument list:
                            Classification::InvalidChars,
                            PoSl::empty(),
                            PoSl::empty(),
                            Some(ix as usize),
                        );
                    }

                    if 0 == ix {
                        if '/' == c {
                            return (
                                // argument list:
                                Classification::SlashRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                                None,
                            );
                        }
                    }

                    if ix == 1 {
                        if '~' == c0 && char_is_path_name_separator_(c1) {
                            if classification_flags::RECOGNISE_TILDE_HOME
                                == (classification_flags::RECOGNISE_TILDE_HOME & parse_flags)
                            {
                                return (
                                    // argument list:
                                    Classification::HomeRooted,
                                    PoSl::new(0, 1),
                                    PoSl::new(1, path.len() - 1),
                                    None,
                                );
                            }
                        }

                        if char_is_drive_letter_(c0) && ':' == c1 {
                            is_drive_2 = true;
                        }


                        if '/' == c0 && '/' == c {

                            // TODO
                            {}
                        } else if '\\' == c0 {

                            return (
                                // argument list:
                                Classification::SlashRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                                None,
                            );
                        }
                    }

                    if ix == 2 {
                        if is_drive_2 {
                            if char_is_path_name_separator_(c) {
                                return (
                                    // argument list:
                                    Classification::DriveLetterRooted,
                                    PoSl::new(0, 3),
                                    PoSl::new(3, path.len() - 3),
                                    None,
                                );
                            } else {
                                return (
                                    // argument list:
                                    Classification::DriveLetterRelative,
                                    PoSl::new(0, 2),
                                    PoSl::new(2, path.len() - 2),
                                    None,
                                );
                            }
                        }
                    }
                }


                if 0 == ix {
                    if '~' == c0 {
                        if classification_flags::RECOGNISE_TILDE_HOME
                            == (classification_flags::RECOGNISE_TILDE_HOME & parse_flags)
                        {
                            return (
                                // argument list:
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                                None,
                            );
                        }
                    }

                    if char_is_path_name_separator_(c0) {
                        return (
                            // argument list:
                            Classification::SlashRooted,
                            PoSl::new(0, 1),
                            PoSl::new(1, path.len() - 1),
                            None,
                        );
                    }
                }


                (
                    // argument list:
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
                    None,
                )
            }

            /// Evaluates whether a character is invalid in a path entry
            /// name.
            /*
            fn char_is_invalid_in_entryname_(c : char) -> bool {
                match c {
                    '\u{0001}'..='\u{001f}' => true,
                    '"' => true,
                    '*' => true,
                    '/' => true,
                    '<' => true,
                    '>' => true,
                    '?' => true,
                    '\\' => true,
                    '|' => true,

                    _ => false,
                }
            }
             */

            /// Evaluates whether a character is invalid in a path.
            fn char_is_invalid_in_path_(c : char) -> bool {
                match c {
                    '\u{0001}'..='\u{001f}' => true,
                    '"' => true,
                    '*' => true,
                    '<' => true,
                    '>' => true,
                    '?' => true,
                    '|' => true,

                    _ => false,
                }
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

            fn str_begins_with_drive_spec_(s : &str) -> bool {
                if s.len() < 2 {
                    return false;
                }

                let c0 = s.as_bytes()[0] as char;
                let c1 = s.as_bytes()[1] as char;

                if c1 != ':' {
                    return false;
                }

                if !char_is_drive_letter_(c0) {
                    return false;
                }

                true
            }

            fn str_is_drive_spec_(s : &str) -> bool {
                if 2 != s.len() {
                    return false;
                }

                str_begins_with_drive_spec_(s)
            }


            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]

                use super::*;

                use fastparse::fastparse::types::PositionalSlice as PoSl;


                #[test]
                fn TEST_char_is_drive_letter__1() {
                    assert!(char_is_drive_letter_('A'));
                    assert!(char_is_drive_letter_('C'));
                    assert!(char_is_drive_letter_('Z'));
                    assert!(char_is_drive_letter_('a'));
                    assert!(char_is_drive_letter_('c'));
                    assert!(char_is_drive_letter_('z'));

                    assert!(!char_is_drive_letter_('.'));
                    assert!(!char_is_drive_letter_('/'));
                    assert!(!char_is_drive_letter_(':'));
                }

                #[test]
                fn TEST_char_is_invalid_in_path__1() {
                    assert!(char_is_invalid_in_path_('<'));
                    assert!(char_is_invalid_in_path_('>'));
                    assert!(char_is_invalid_in_path_('|'));

                    assert!(!char_is_invalid_in_path_('-'));
                    assert!(!char_is_invalid_in_path_('/'));
                    assert!(!char_is_invalid_in_path_(':'));
                    assert!(!char_is_invalid_in_path_(';'));
                    assert!(!char_is_invalid_in_path_('\\'));
                    assert!(!char_is_invalid_in_path_('a'));
                }

                #[test]
                fn TEST_char_is_path_name_separator__1() {
                    assert!(char_is_path_name_separator_('/'));
                    assert!(char_is_path_name_separator_('\\'));

                    assert!(!char_is_path_name_separator_('-'));
                    assert!(!char_is_path_name_separator_(':'));
                    assert!(!char_is_path_name_separator_(';'));
                    assert!(!char_is_path_name_separator_('a'));
                }

                #[test]
                fn TEST_classify_root__1() {
                    let test_criteria = &[
                        (r"abc", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),

                        (r"abc.def", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 7), None),

                        (r"/", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),
                        (r"\", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),

                        (r"/abc", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 3), None),
                        (r"\abc", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 3), None),

                        (r"/abc.def", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 7), None),
                        (r"\abc.def", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 7), None),

                        (r"C:/dir/sub-dir/file.ext", 0, Classification::DriveLetterRooted, PoSl::new(0, 3), PoSl::new(3, 20), None),
                        (r"C:\dir\sub-dir\file.ext", 0, Classification::DriveLetterRooted, PoSl::new(0, 3), PoSl::new(3, 20), None),

                        (r"C:dir/sub-dir/file.ext", 0, Classification::DriveLetterRelative, PoSl::new(0, 2), PoSl::new(2, 20), None),
                        (r"C:dir\sub-dir\file.ext", 0, Classification::DriveLetterRelative, PoSl::new(0, 2), PoSl::new(2, 20), None),

                        ("abc", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),
                        ("abc.def", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 7), None),
                        ("/", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),
                        ("/abc", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 3), None),
                        ("/abc.def", 0, Classification::SlashRooted, PoSl::new(0, 1), PoSl::new(1, 7), None),
                        ("~", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 1), None),
                        ("~", classification_flags::RECOGNISE_TILDE_HOME, Classification::HomeRooted, PoSl::new(0, 1), PoSl::new(1, 0), None),
                        ("~/", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~/", classification_flags::RECOGNISE_TILDE_HOME, Classification::HomeRooted, PoSl::new(0, 1), PoSl::new(1, 1), None),
                        ("~a", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~a", classification_flags::RECOGNISE_TILDE_HOME, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 2), None),
                        ("~a/", 0, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),
                        ("~a/", classification_flags::RECOGNISE_TILDE_HOME, Classification::Relative, PoSl::new(0, 0), PoSl::new(0, 3), None),

                        ("|a", 0, Classification::InvalidChars, PoSl::empty(), PoSl::empty(), Some(0)),
                        ("a|", 0, Classification::InvalidChars, PoSl::empty(), PoSl::empty(), Some(1)),
                    ];

                    for (
                        path,
                        parse_flags,
                        expected_classification,
                        expected_root,
                        expected_path_root_stripped,
                        expected_first_bad_char_index,
                    ) in test_criteria
                    {
                        let (classification, root, path_root_stripped, first_bad_char_index) =
                            classify_root_(*path, *parse_flags);

                        assert_eq!(*expected_classification, classification, "wrong classification {classification:?} ({expected_classification:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_root, root, "wrong root {root:?} ({expected_root:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_path_root_stripped, path_root_stripped, "wrong path-root-stripped {path_root_stripped:?} ({expected_path_root_stripped:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                        assert_eq!(*expected_first_bad_char_index, first_bad_char_index, "wrong first-bad-char-index {first_bad_char_index:?} ({expected_first_bad_char_index:?} expected) when parsing '{path}' with flags 0x{parse_flags:08x}");
                    }
                }

                #[test]
                fn TEST_count_parts__1() {
                    let test_criteria = &[
                        (r"", 0, 0, 0),
                        (r"a", 0, 0, 0),
                        (r"a\", 0, 1, 0),
                        (r"abc", 0, 0, 0),
                        (r"abc\", 0, 1, 0),
                        (r"abc\def", 0, 1, 0),
                        (r"abc\def\", 0, 2, 0),
                        (r"🐻", 0, 0, 0),
                        (r"🐻\", 0, 1, 0),
                        (r"🐻\🐻‍❄️", 0, 1, 0),
                        (r"🐻\🐻‍❄️\", 0, 2, 0),
                        (r"\", 0, 1, 0),
                        (r"\abc", 0, 1, 0),
                        (r"\abc\", 0, 2, 0),
                        (r"\abc\def", 0, 2, 0),
                        (r"\abc\def\", 0, 3, 0),
                        (r"..", 0, 0, 0),
                        (r"..\", 0, 1, 1),
                        (r"\..", 0, 1, 0),
                        (r"\..\", 0, 2, 1),

                        ("", 0, 0, 0),
                        ("a", 0, 0, 0),
                        ("a/", 0, 1, 0),
                        ("abc", 0, 0, 0),
                        ("abc/", 0, 1, 0),
                        ("abc/def", 0, 1, 0),
                        ("abc/def/", 0, 2, 0),
                        ("🐻", 0, 0, 0),
                        ("🐻/", 0, 1, 0),
                        ("🐻/🐻‍❄️", 0, 1, 0),
                        ("🐻/🐻‍❄️/", 0, 2, 0),
                        ("/", 0, 1, 0),
                        ("/abc", 0, 1, 0),
                        ("/abc/", 0, 2, 0),
                        ("/abc/def", 0, 2, 0),
                        ("/abc/def/", 0, 3, 0),
                        ("..", 0, 0, 0),
                        ("../", 0, 1, 1),
                        ("/..", 0, 1, 0),
                        ("/../", 0, 2, 1),
                    ];

                    for (s, parse_flags, expected_number_of_parts, expected_number_of_dots_parts) in test_criteria {
                        let (number_of_parts, number_of_dots_parts) = count_parts_(*s, *parse_flags);

                        assert_eq!(*expected_number_of_parts, number_of_parts, "wrong number of parts {number_of_parts} ({expected_number_of_parts} expected) when parsing '{s}'");
                        assert_eq!(*expected_number_of_dots_parts, number_of_dots_parts, "wrong number of dots parts {number_of_dots_parts} ({expected_number_of_dots_parts} expected) when parsing '{s}'");
                    }
                }

                #[test]
                fn TEST_find_last_slash__1() {
                }

                #[test]
                fn TEST_unc_split_() {
                    {
                        let input = "";
                        let parse_flags = 0;
                        let expected : Vec<&str> = vec![];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"C:\Test\Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"C:",
                            r"\",
                            r"Test",
                            r"\",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"C:/Test/Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"C:",
                            r"/",
                            r"Test",
                            r"/",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"C:Test\Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"C:",
                            r"Test",
                            r"\",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = "COM1";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            "COM1",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\.\COM1";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r".",
                            r"\",
                            r"COM1",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"COM1\";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"COM1",
                            r"\",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\.\Volume{b75e2c83-0000-0000-0000-602f00000000}\Test\Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r".",
                            r"\",
                            r"Volume{b75e2c83-0000-0000-0000-602f00000000}",
                            r"\",
                            r"Test",
                            r"\",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\.\Volume{b75e2c83-0000-0000-0000-602f00000000}/Test/Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r".",
                            r"\",
                            r"Volume{b75e2c83-0000-0000-0000-602f00000000}",
                            r"/",
                            r"Test",
                            r"/",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\?\C:/Test/Foo.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r"?",
                            r"\",
                            r"C:",
                            r"/",
                            r"Test",
                            r"/",
                            r"Foo.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\.\UNC\LOCALHOST\c$\temp\test-file.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r".",
                            r"\",
                            r"UNC",
                            r"\",
                            r"LOCALHOST",
                            r"\",
                            r"c$",
                            r"\",
                            r"temp",
                            r"\",
                            r"test-file.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\.\UNC\127.0.0.1\c$\temp\test-file.txt";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r".",
                            r"\",
                            r"UNC",
                            r"\",
                            r"127.0.0.1",
                            r"\",
                            r"c$",
                            r"\",
                            r"temp",
                            r"\",
                            r"test-file.txt",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\127.0.0.1\c$\dir\stem.ext";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r"127.0.0.1",
                            r"\",
                            r"c$",
                            r"\",
                            r"dir",
                            r"\",
                            r"stem.ext",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\127.0.0.1\c$\dir\stem.ext";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r"127.0.0.1",
                            r"\",
                            r"c$",
                            r"\",
                            r"dir",
                            r"\",
                            r"stem.ext",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }

                    {
                        let input = r"\\?\server1\utilities\\filecomparer\";
                        let parse_flags = 0;
                        let expected = vec![
                            // element list:
                            r"\\",
                            r"?",
                            r"\",
                            r"server1",
                            r"\",
                            r"utilities",
                            r"\\",
                            r"filecomparer",
                            r"\",
                        ];
                        let actual = unc_split_(input, parse_flags);

                        assert_eq!(expected, actual);
                    }
                }
            }
        }
    }
}


#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::libpath::util::common::ClassificationResult;

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
        fn TEST_path_classify_WITH_EMPTY_INPUT() {
            let flag_max = 0 | IGNORE_SLASH_RUNS | IGNORE_INVALID_CHARS | RECOGNISE_TILDE_HOME;

            for flags in 0..=flag_max {
                let (cl, cr) = path_classify("", flags);

                assert_eq!(Classification::Empty, cl);

                assert_eq!(ClassificationResult::empty(), cr);
            }
        }

        #[test]
        fn TEST_path_classify_WITH_EntryName_ONLY() {
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("ab.", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("ab.", cr.EntryName.substring_of(path));
                assert_eq!("ab.", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("a..", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("a..", cr.EntryName.substring_of(path));
                assert_eq!("a..", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("...", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("...", cr.EntryName.substring_of(path));
                assert_eq!("...", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_RELATIVE_Directory_AND_EntryName() {
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
        fn TEST_path_classify_WITH_RELATIVE_Directory_ONLY() {
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

                assert_eq!("dir1/dir2/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir1/dir2/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir1/dir2/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!("dir1/../", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir1/../", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir1/../", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!("../dir1/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("../dir1/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("../dir1/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!(".././", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(".././", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(".././", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "dir-1/../././././././././././abc/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 33), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 33), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 33), cr.Directory);
                assert_eq!(13, cr.NumDirectoryParts);
                assert_eq!(11, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(33, 0), cr.EntryName);
                assert_eq!(PoSl::new(33, 0), cr.Stem);
                assert_eq!(PoSl::new(33, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("dir-1/../././././././././././abc/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir-1/../././././././././././abc/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir-1/../././././././././././abc/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS1_ONLY_INTERPRETED_AS_Stem() {
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

            assert_eq!(".", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!(".", cr.EntryName.substring_of(path));
            assert_eq!(".", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
    }

        #[test]
        fn TEST_path_classify_WITH_DOTS2_ONLY_INTERPRETED_AS_Stem() {
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

            assert_eq!("..", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("..", cr.EntryName.substring_of(path));
            assert_eq!("..", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS3_ONLY_INTERPRETED_AS_Stem() {
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
            assert_eq!(PoSl::new(0, 3), cr.Stem);
            assert_eq!(PoSl::new(3, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("...", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("...", cr.EntryName.substring_of(path));
            assert_eq!("...", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS4_ONLY_INTERPRETED_AS_Stem() {
            let path = "....";
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

            assert_eq!("....", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("....", cr.EntryName.substring_of(path));
            assert_eq!("....", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_SLASH_ROOT_ONLY() {
            let path = "/";
            let parse_flags : i32 = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::SlashRooted, cl);

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

            assert_eq!("/", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("/", cr.Location.substring_of(path));
            assert_eq!("/", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("", cr.EntryName.substring_of(path));
            assert_eq!("", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_SLASHROOTED_PATH() {
            {
                let path = "/dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::new(0, 1), cr.Root);
                assert_eq!(PoSl::new(1, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.EntryName);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("/", cr.Root.substring_of(path));
                assert_eq!("dir/sub-dir/", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }

            {
                let path = "/dir-1/../././././././././././abc";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 33), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 30), cr.Location);
                assert_eq!(PoSl::new(0, 1), cr.Root);
                assert_eq!(PoSl::new(1, 29), cr.Directory);
                assert_eq!(12, cr.NumDirectoryParts);
                assert_eq!(11, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(30, 3), cr.EntryName);
                assert_eq!(PoSl::new(30, 3), cr.Stem);
                assert_eq!(PoSl::new(33, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("/dir-1/../././././././././././abc", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("/dir-1/../././././././././././", cr.Location.substring_of(path));
                assert_eq!("/", cr.Root.substring_of(path));
                assert_eq!("dir-1/../././././././././././", cr.Directory.substring_of(path));
                assert_eq!("abc", cr.EntryName.substring_of(path));
                assert_eq!("abc", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_HomeRooted_PATH_WITHOUT__RECOGNISE_TILDE_HOME() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 14), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.EntryName);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.EntryName.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_HomeRooted_PATH_WITH__RECOGNISE_TILDE_HOME() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags = RECOGNISE_TILDE_HOME;
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
        fn TEST_path_classify_WITH_HOME_ONLY_WITHOUT__RECOGNISE_TILDE_HOME() {
            let path = "~";
            let parse_flags = 0;
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

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("~", cr.EntryName.substring_of(path));
            assert_eq!("~", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_HOME_ONLY_WITH__RECOGNISE_TILDE_HOME() {
            let path = "~";
            let parse_flags = RECOGNISE_TILDE_HOME;
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
        fn TEST_path_classify_WITH_EMPTY_INPUT() {
            let flag_max =
                0 | IGNORE_SLASH_RUNS | IGNORE_INVALID_CHARS | RECOGNISE_TILDE_HOME | IGNORE_INVALID_CHARS_IN_LONG_PATH;

            for flags in 0..=flag_max {
                let (cl, cr) = path_classify("", flags);

                assert_eq!(Classification::Empty, cl);

                assert_eq!(ClassificationResult::empty(), cr);
            }
        }

        #[test]
        fn TEST_path_classify_WITH_EntryName_ONLY() {
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("ab.", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("ab.", cr.EntryName.substring_of(path));
                assert_eq!("ab.", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("a..", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("a..", cr.EntryName.substring_of(path));
                assert_eq!("a..", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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
                assert_eq!(PoSl::new(0, 3), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("...", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("...", cr.EntryName.substring_of(path));
                assert_eq!("...", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_RELATIVE_Directory_AND_EntryName() {
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

                assert_eq!("dir/name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir/", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.EntryName.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
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

                assert_eq!(r"dir\name.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"dir\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"dir\", cr.Directory.substring_of(path));
                assert_eq!("name.ext", cr.EntryName.substring_of(path));
                assert_eq!("name", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_RELATIVE_Directory_ONLY() {
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

                assert_eq!("dir1/dir2/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir1/dir2/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir1/dir2/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!("dir1/../", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("dir1/../", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("dir1/../", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!("../dir1/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("../dir1/", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!("../dir1/", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!(".././", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(".././", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(".././", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!(r"dir\", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"dir\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"dir\", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
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

                assert_eq!(r"dir1\dir2\", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"dir1\dir2\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"dir1\dir2\", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = r"dir1\\dir2\";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 11), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 11), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 11), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(11, 0), cr.EntryName);
                assert_eq!(PoSl::new(11, 0), cr.Stem);
                assert_eq!(PoSl::new(11, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"dir1\\dir2\", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"dir1\\dir2\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"dir1\\dir2\", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = r"dir1\dir2\\";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::Relative, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 11), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 11), cr.Location);
                assert_eq!(PoSl::empty(), cr.Root);
                assert_eq!(PoSl::new(0, 11), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(11, 0), cr.EntryName);
                assert_eq!(PoSl::new(11, 0), cr.Stem);
                assert_eq!(PoSl::new(11, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"dir1\dir2\\", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"dir1\dir2\\", cr.Location.substring_of(path));
                assert_eq!("", cr.Root.substring_of(path));
                assert_eq!(r"dir1\dir2\\", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS1_ONLY_INTERPRETED_AS_Stem() {
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

            assert_eq!(".", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!(".", cr.EntryName.substring_of(path));
            assert_eq!(".", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
    }

        #[test]
        fn TEST_path_classify_WITH_DOTS2_ONLY_INTERPRETED_AS_Stem() {
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

            assert_eq!("..", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("..", cr.EntryName.substring_of(path));
            assert_eq!("..", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS3_ONLY_INTERPRETED_AS_Stem() {
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
            assert_eq!(PoSl::new(0, 3), cr.Stem);
            assert_eq!(PoSl::new(3, 0), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("...", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("...", cr.EntryName.substring_of(path));
            assert_eq!("...", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_DOTS4_ONLY_INTERPRETED_AS_Stem() {
            let path = "....";
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

            assert_eq!("....", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("....", cr.EntryName.substring_of(path));
            assert_eq!("....", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_DRIVE_ROOT_ONLY() {
            {
                let path = r"C:\";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 3), cr.Location);
                assert_eq!(PoSl::new(0, 3), cr.Root);
                assert_eq!(PoSl::new(3, 0), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(3, 0), cr.EntryName);
                assert_eq!(PoSl::new(3, 0), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:\", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:\", cr.Location.substring_of(path));
                assert_eq!(r"C:\", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }

            {
                let path = "C:/";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 3), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 3), cr.Location);
                assert_eq!(PoSl::new(0, 3), cr.Root);
                assert_eq!(PoSl::new(3, 0), cr.Directory);
                assert_eq!(0, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(3, 0), cr.EntryName);
                assert_eq!(PoSl::new(3, 0), cr.Stem);
                assert_eq!(PoSl::new(3, 0), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:/", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:/", cr.Location.substring_of(path));
                assert_eq!("C:/", cr.Root.substring_of(path));
                assert_eq!("", cr.Directory.substring_of(path));
                assert_eq!("", cr.EntryName.substring_of(path));
                assert_eq!("", cr.Stem.substring_of(path));
                assert_eq!("", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_SLASHROOTED_PATH() {
            {
                let path = "/dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::SlashRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 21), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 13), cr.Location);
                assert_eq!(PoSl::new(0, 1), cr.Root);
                assert_eq!(PoSl::new(1, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.EntryName);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("/", cr.Root.substring_of(path));
                assert_eq!("dir/sub-dir/", cr.Directory.substring_of(path));
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
                assert_eq!(PoSl::new(0, 1), cr.Root);
                assert_eq!(PoSl::new(1, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(13, 8), cr.EntryName);
                assert_eq!(PoSl::new(13, 4), cr.Stem);
                assert_eq!(PoSl::new(17, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!(r"\", cr.Root.substring_of(path));
                assert_eq!(r"dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_DriveLetterRooted_PATH() {
            {
                let path = "C:/dir/sub-dir/file.ext";
                let parse_flags : i32 = 0;
                let (cl, cr) = path_classify(path, parse_flags);

                assert_eq!(Classification::DriveLetterRooted, cl);

                assert_ne!(ClassificationResult::empty(), cr);
                assert_eq!(PoSl::new(0, 23), cr.Input);
                assert_eq!(PoSl::empty(), cr.Prefix);
                assert_eq!(PoSl::new(0, 15), cr.Location);
                assert_eq!(PoSl::new(0, 3), cr.Root);
                assert_eq!(PoSl::new(3, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.EntryName);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!("C:/dir/sub-dir/file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!("C:/dir/sub-dir/", cr.Location.substring_of(path));
                assert_eq!("C:/", cr.Root.substring_of(path));
                assert_eq!("dir/sub-dir/", cr.Directory.substring_of(path));
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
                assert_eq!(PoSl::new(0, 3), cr.Root);
                assert_eq!(PoSl::new(3, 12), cr.Directory);
                assert_eq!(2, cr.NumDirectoryParts);
                assert_eq!(0, cr.NumDotsDirectoryParts);
                assert_eq!(PoSl::new(15, 8), cr.EntryName);
                assert_eq!(PoSl::new(15, 4), cr.Stem);
                assert_eq!(PoSl::new(19, 4), cr.Extension);
                assert!(cr.FirstInvalid.is_empty());

                assert_eq!(r"C:\dir\sub-dir\file.ext", cr.Input.substring_of(path));
                assert_eq!("", cr.Prefix.substring_of(path));
                assert_eq!(r"C:\dir\sub-dir\", cr.Location.substring_of(path));
                assert_eq!(r"C:\", cr.Root.substring_of(path));
                assert_eq!(r"dir\sub-dir\", cr.Directory.substring_of(path));
                assert_eq!("file.ext", cr.EntryName.substring_of(path));
                assert_eq!("file", cr.Stem.substring_of(path));
                assert_eq!(".ext", cr.Extension.substring_of(path));
            }
        }

        #[test]
        fn TEST_path_classify_WITH_DriveLetterRelative_PATH() {
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
        fn TEST_path_classify_WITH_HomeRooted_PATH_WITH__RECOGNISE_TILDE_HOME() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags = RECOGNISE_TILDE_HOME;
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
        fn TEST_path_classify_WITH_HomeRooted_PATH_WITHOUT__RECOGNISE_TILDE_HOME() {
            let path = "~/dir/sub-dir/file.ext";
            let parse_flags = 0;
            let (cl, cr) = path_classify(path, parse_flags);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 22), cr.Input);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::new(0, 14), cr.Location);
            assert_eq!(PoSl::empty(), cr.Root);
            assert_eq!(PoSl::new(0, 14), cr.Directory);
            assert_eq!(3, cr.NumDirectoryParts);
            assert_eq!(0, cr.NumDotsDirectoryParts);
            assert_eq!(PoSl::new(14, 8), cr.EntryName);
            assert_eq!(PoSl::new(14, 4), cr.Stem);
            assert_eq!(PoSl::new(18, 4), cr.Extension);
            assert!(cr.FirstInvalid.is_empty());

            assert_eq!("~/dir/sub-dir/file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("~/dir/sub-dir/", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.EntryName.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }

        #[test]
        fn TEST_path_classify_WITH_HOME_ONLY_WITH__RECOGNISE_TILDE_HOME() {
            let path = "~";
            let parse_flags = RECOGNISE_TILDE_HOME;
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

        #[test]
        fn TEST_path_classify_WITH_HOME_ONLY_WITHOUT__RECOGNISE_TILDE_HOME() {
            let path = "~";
            let parse_flags = 0;
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

            assert_eq!("~", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("", cr.Directory.substring_of(path));
            assert_eq!("~", cr.EntryName.substring_of(path));
            assert_eq!("~", cr.Stem.substring_of(path));
            assert_eq!("", cr.Extension.substring_of(path));
        }
    }
}


/* ///////////////////////////// end of file //////////////////////////// */

