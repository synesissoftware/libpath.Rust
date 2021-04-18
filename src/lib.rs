
extern crate fastparse;

pub mod libpath {

    pub mod util {

        pub mod common {

            use fastparse::fastparse::types::PositionalSlice as PoSl;

            #[derive(Debug)]
            #[derive(PartialEq, Eq)]
            pub struct ClassificationResult {

                pub Input                   :   PoSl,
                pub FullPath                :   PoSl,
                pub Prefix                  :   PoSl,
                pub Location                :   PoSl,
                pub Root                    :   PoSl,
                pub Directory               :   PoSl,
                pub NumDirectoryParts       :   usize,
                pub NumDotsDirectoryParts   :   usize,
                pub Entry                   :   PoSl,
                pub Stem                    :   PoSl,
                pub Extension               :   PoSl,
                pub FirstInvalid            :   PoSl,
            }

            impl ClassificationResult {

                pub fn empty() -> Self {

                    Self {

                        Input                   :   PoSl::empty(),
                        FullPath                :   PoSl::empty(),
                        Prefix                  :   PoSl::empty(),
                        Location                :   PoSl::empty(),
                        Root                    :   PoSl::empty(),
                        Directory               :   PoSl::empty(),
                        NumDirectoryParts       :   0usize,
                        NumDotsDirectoryParts   :   0usize,
                        Entry                   :   PoSl::empty(),
                        Stem                    :   PoSl::empty(),
                        Extension               :   PoSl::empty(),
                        FirstInvalid            :   PoSl::empty(),
                    }
                }
            }
        }

        pub mod windows {

            pub const IGNORE_SLASH_RUNS : i32                   =   0x00000001;
            pub const IGNORE_INVALID_CHARS : i32                =   0x00000002;
            pub const RECOGNISE_SET_TILE_HOME : i32             =   0x00000004;
            pub const IGNORE_INVALID_CHARS_IN_LONG_PATH : i32   =   0x00000002;

            #[derive(Debug, PartialEq)]
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

            use super::common::ClassificationResult;
            use fastparse::fastparse::types::PositionalSlice as PoSl;

            pub fn path_classify(
                path        :   &str,
                parse_flags :   i32,
            ) -> (
                Classification,
                ClassificationResult,
            ) {

                if path.is_empty() {

                    return (Classification::Empty, ClassificationResult::empty());
                }

                let mut cr = ClassificationResult::empty();

                cr.Input = PoSl::new(0, path.len());

                let (r_cl, volume, path_vol_stripped) = classify_root_(path, parse_flags);

                // now search within volume-stripped path

                let last_slash = find_last_slash_(path);

                match last_slash {

                    Some(index) => {

                        // if there's a slash, then there is a directory and, potentially, an entry


                        let dir_len = index + 1;

                        cr.Directory = PoSl::new(volume.len(), dir_len);

                        let (num_parts, num_dir_parts) = count_parts_(cr.Directory.substring_of(path), parse_flags);
                        cr.NumDirectoryParts = num_parts;
                        cr.NumDotsDirectoryParts = num_dir_parts;

                        cr.Entry = PoSl::new(volume.len() + dir_len, path_vol_stripped.len() - dir_len);
                    }
                    None => {

                        // if there's no slash, then the while (stripped) path is the entry


                        cr.Entry = PoSl::new(volume.len(), path_vol_stripped.len());
                    }
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
                        }
                        None => {

                            cr.Stem = cr.Entry;
                            cr.Extension = PoSl::new(cr.Entry.offset + cr.Entry.length, 0);
                        }
                    }
                }

                (r_cl, cr)
            }

            pub fn classify_root_(
                path        :   &str,
                parse_flags :   i32,

            ) -> (
                Classification,
                PoSl,               // volume
                PoSl,               // path_volume_stripped
            ) {

                let mut ssi1 = PoSl::new(0, path.len());

                return (
                    Classification::Relative,
                    PoSl::empty(),
                    ssi1,
                );
            }

            fn char_is_path_name_separator_(c : char) -> bool {

                match c {

                    '/' => true,
                    '\\' => true,
                    _ => false,
                }
            }

            fn find_last_slash_(s: &str) -> Option<usize> {

                // TODO: consider rfind(&['/', '\\'][..])

                let last_fslash = s.rfind('/');

                match last_fslash {

                    Some(index1) => {

                        let last_rslash = s[index1+1..].rfind('\\');

                        match last_rslash {

                            Some(index2) => {

                                return if index2 > index1 {

                                    Some(index2)
                                } else {

                                    Some(index1)
                                }
                            },
                            None => {

                                return last_fslash;
                            },
                        }
                    },
                    None => {

                        return s.rfind('\\');
                    },
                }
            }

            fn count_parts_(
                s : &str,
                parse_flags : i32,
            ) -> (
                usize,  // number of parts
                usize,  // number of dots parts
            )
            {
                // This function counts the number of directory parts and the
                // number of those that are dots directories

                let mut np = 0usize;
                let mut nd = 0usize;

                let mut prev = 'X';

                let mut num_dots = 0;

                let mut ix = -1;

                for c in s.chars() {

                    ix += 1;

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
                    } else
                    {
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
        }
    }
}

#[cfg(test)]
mod tests {

    use crate ::fastparse;
    use crate ::libpath;

    use fastparse::fastparse::types::PositionalSlice as PoSl;
    use libpath::util::common::{

        ClassificationResult,
    };

    #[test]
    fn windows_path_classify_empty()  {

        use libpath::util::windows::{

            *,
        };

        let flag_max    =   0
                        |   IGNORE_SLASH_RUNS
                        |   IGNORE_INVALID_CHARS
                        |   RECOGNISE_SET_TILE_HOME
                        |   IGNORE_INVALID_CHARS_IN_LONG_PATH
                        ;
            
        for flags in 0..=flag_max {

            let (cl, cr) = libpath::util::windows::path_classify("", flags);

            assert_eq!(Classification::Empty, cl);

            assert_eq!(ClassificationResult::empty(), cr);
        }
    }

    #[test]
    fn windows_path_classify_entry_only() {

        use libpath::util::windows::{

            *,
        };

        {
            let (cl, cr) = libpath::util::windows::path_classify("name.ext", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 8), cr.FullPath);
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
        }

        {
            let (cl, cr) = libpath::util::windows::path_classify("name", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
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
            let (cl, cr) = libpath::util::windows::path_classify(".ext", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
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
            let (cl, cr) = libpath::util::windows::path_classify("ab.", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 3), cr.Input);
            // assert_eq!(PoSl::new(0, 3), cr.FullPath);
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
            let (cl, cr) = libpath::util::windows::path_classify("a..", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 3), cr.Input);
            // assert_eq!(PoSl::new(0, 3), cr.FullPath);
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
            let (cl, cr) = libpath::util::windows::path_classify("...", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 3), cr.Input);
            // assert_eq!(PoSl::new(0, 3), cr.FullPath);
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

        use libpath::util::windows::{

            *,
        };

        let (cl, cr) = libpath::util::windows::path_classify("dir/name.ext", 0);

        assert_eq!(Classification::Relative, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 12), cr.Input);
        // assert_eq!(PoSl::new(0, 12), cr.FullPath);
        assert_eq!(PoSl::empty(), cr.Prefix);
        assert_eq!(PoSl::empty(), cr.Location);
        assert_eq!(PoSl::empty(), cr.Root);
        assert_eq!(PoSl::new(0, 4), cr.Directory);
        assert_eq!(1, cr.NumDirectoryParts);
        assert_eq!(0, cr.NumDotsDirectoryParts);
        assert_eq!(PoSl::new(4, 8), cr.Entry);
        assert_eq!(PoSl::new(4, 4), cr.Stem);
        assert_eq!(PoSl::new(8, 4), cr.Extension);
        assert!(cr.FirstInvalid.is_empty());
    }

    #[test]
    fn windows_path_classify_rel_dir_only() {

        use libpath::util::windows::{

            *,
        };

        {
            let (cl, cr) = libpath::util::windows::path_classify("dir/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify("dir1/dir2/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 10), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify("dir1/../", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify("../dir1/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify(".././", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 5), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify("dir\\", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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
            let (cl, cr) = libpath::util::windows::path_classify("dir1\\dir2\\", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 10), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
            assert_eq!(PoSl::empty(), cr.Prefix);
            assert_eq!(PoSl::empty(), cr.Location);
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

        use libpath::util::windows::{

            *,
        };

        let (cl, cr) = libpath::util::windows::path_classify(".", 0);

        assert_eq!(Classification::Relative, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 1), cr.Input);
        // assert_eq!(PoSl::new(0, 1), cr.FullPath);
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

        use libpath::util::windows::{

            *,
        };

        let (cl, cr) = libpath::util::windows::path_classify("..", 0);

        assert_eq!(Classification::Relative, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 2), cr.Input);
        // assert_eq!(PoSl::new(0, 2), cr.FullPath);
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
}
