
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

        pub mod unix {

            pub const IGNORE_SLASH_RUNS : i32           =   0x00000001;
            pub const IGNORE_INVALID_CHARS : i32        =   0x00000002;
            pub const RECOGNISE_SET_TILDE_HOME : i32    =   0x00000004;

            #[derive(Debug, PartialEq)]
            pub enum Classification {

                InvalidSlashRuns = -3,
                InvalidChars = -2,
                Invalid = -1,
                Unknown,
                Empty,
                Relative,
                SlashRooted,
                Reserved1,
                Reserved2,
                Reserved3,
                Reserved4,
                HomeRooted,
            }

            use super::common::ClassificationResult;
            use fastparse::fastparse::types::PositionalSlice as PoSl;

            pub fn path_classify(
                path        :   &str,
                parse_flags :   i32,
            ) -> (Classification, ClassificationResult) {

                if path.is_empty() {

                    return (Classification::Empty, ClassificationResult::empty());
                }

                let mut cl = Classification::Unknown;
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
                    }
                    None => {

                        // if there's no slash, then the while (stripped) path is the entry

                        cr.Entry = PoSl::new(root.len(), path_root_stripped.len());
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

                cr.Location = PoSl::new(0, cr.Entry.offset);

                (cl, cr)
            }

            pub fn classify_root_(
                path        :   &str,
                parse_flags :   i32,
            ) -> (
                Classification,
                PoSl,               // root
                PoSl,               // path_root_stripped
            ) {

                debug_assert!(!path.is_empty());

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
                                Classification::SlashRooted,
                                PoSl::empty(),
                                PoSl::new(0, path.len()),
                            );
                        }

                        if 1 == ix {

                            return (
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                            );
                        }
                    }

                    break;
                }

                return (
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
                );
            }

            fn char_is_path_name_separator_(c : char) -> bool {

                c == '/'
            }

            fn find_last_slash_(s: &str) -> Option<usize> {

                s.rfind('/')
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

        pub mod windows {

            pub const IGNORE_SLASH_RUNS : i32                   =   0x00000001;
            pub const IGNORE_INVALID_CHARS : i32                =   0x00000002;
            pub const RECOGNISE_SET_TILDE_HOME : i32            =   0x00000004;
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
                    }
                    None => {

                        // if there's no slash, then the while (stripped) path is the entry

                        cr.Entry = PoSl::new(root.len(), path_root_stripped.len());
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

                cr.Location = PoSl::new(0, cr.Entry.offset);

                (cl, cr)
            }

            pub fn classify_root_(
                path        :   &str,
                parse_flags :   i32,
            ) -> (
                Classification,
                PoSl,               // root
                PoSl,               // path_root_stripped
            ) {

                debug_assert!(!path.is_empty());

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
                                Classification::SlashRooted,
                                PoSl::empty(),
                                PoSl::new(0, path.len()),
                            );
                        }

                        if 1 == ix {

                            return (
                                Classification::HomeRooted,
                                PoSl::new(0, 1),
                                PoSl::new(1, path.len() - 1),
                            );
                        }
                    }

                    break;
                }

                return (
                    Classification::Relative,
                    PoSl::empty(),
                    PoSl::new(0, path.len()),
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
    fn unix_path_classify_empty()  {

        use libpath::util::unix::{

            *,
        };

        let flag_max    =   0
                        |   IGNORE_SLASH_RUNS
                        |   IGNORE_INVALID_CHARS
                        |   RECOGNISE_SET_TILDE_HOME
                        ;

        for flags in 0..=flag_max {

            let (cl, cr) = path_classify("", flags);

            assert_eq!(Classification::Empty, cl);

            assert_eq!(ClassificationResult::empty(), cr);
        }
    }

    #[test]
    fn windows_path_classify_empty()  {

        use libpath::util::windows::{

            *,
        };

        let flag_max    =   0
                        |   IGNORE_SLASH_RUNS
                        |   IGNORE_INVALID_CHARS
                        |   RECOGNISE_SET_TILDE_HOME
                        |   IGNORE_INVALID_CHARS_IN_LONG_PATH
                        ;

        for flags in 0..=flag_max {

            let (cl, cr) = path_classify("", flags);

            assert_eq!(Classification::Empty, cl);

            assert_eq!(ClassificationResult::empty(), cr);
        }
    }

    #[test]
    fn unix_path_classify_entry_only() {

        use libpath::util::unix::{

            *,
        };

        {
            let path = "name.ext";
            let (cl, cr) = path_classify(path, 0);

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
            let (cl, cr) = path_classify("name", 0);

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
            let (cl, cr) = path_classify(".ext", 0);

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
            let (cl, cr) = path_classify("ab.", 0);

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
            let (cl, cr) = path_classify("a..", 0);

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
            let (cl, cr) = path_classify("...", 0);

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
    fn windows_path_classify_entry_only() {

        use libpath::util::windows::{

            *,
        };

        {
            let path = "name.ext";
            let (cl, cr) = path_classify(path, 0);

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
            let (cl, cr) = path_classify("name", 0);

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
            let (cl, cr) = path_classify(".ext", 0);

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
            let (cl, cr) = path_classify("ab.", 0);

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
            let (cl, cr) = path_classify("a..", 0);

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
            let (cl, cr) = path_classify("...", 0);

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
    fn unix_path_classify_rel_dir_and_name() {

        use libpath::util::unix::{

            *,
        };

        let path = "dir/name.ext";
        let (cl, cr) = path_classify(path, 0);

        assert_eq!(Classification::Relative, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 12), cr.Input);
        // assert_eq!(PoSl::new(0, 12), cr.FullPath);
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
    fn windows_path_classify_rel_dir_and_name() {

        use libpath::util::windows::{

            *,
        };

        {
            let (cl, cr) = path_classify("dir/name.ext", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 12), cr.Input);
            // assert_eq!(PoSl::new(0, 12), cr.FullPath);
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
            let (cl, cr) = path_classify("dir\\name.ext", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 12), cr.Input);
            // assert_eq!(PoSl::new(0, 12), cr.FullPath);
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
    fn unix_path_classify_rel_dir_only() {

        use libpath::util::unix::{

            *,
        };

        {
            let path = "dir/";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
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
            let (cl, cr) = path_classify("dir1/dir2/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 10), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify("dir1/../", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify("../dir1/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify(".././", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 5), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
    fn windows_path_classify_rel_dir_only() {

        use libpath::util::windows::{

            *,
        };

        {
            let path = "dir/";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
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
            let (cl, cr) = path_classify("dir1/dir2/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 10), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify("dir1/../", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify("../dir1/", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 8), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify(".././", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 5), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
            let (cl, cr) = path_classify("dir\\", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 4), cr.Input);
            // assert_eq!(PoSl::new(0, 4), cr.FullPath);
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
            let (cl, cr) = path_classify("dir1\\dir2\\", 0);

            assert_eq!(Classification::Relative, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 10), cr.Input);
            // assert_eq!(PoSl::new(0, 10), cr.FullPath);
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
    fn unix_path_classify_dots1_only() {

        use libpath::util::unix::{

            *,
        };

        let (cl, cr) = path_classify(".", 0);

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
    fn windows_path_classify_dots1_only() {

        use libpath::util::windows::{

            *,
        };

        let (cl, cr) = path_classify(".", 0);

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
    fn unix_path_classify_dots2_only() {

        use libpath::util::unix::{

            *,
        };

        let (cl, cr) = path_classify("..", 0);

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

    #[test]
    fn windows_path_classify_dots2_only() {

        use libpath::util::windows::{

            *,
        };

        let (cl, cr) = path_classify("..", 0);

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

    #[test]
    fn unix_path_classify_slashrooted_path() {

        use libpath::util::unix::{

            *,
        };

        let path = "/dir/sub-dir/file.ext";
        let (cl, cr) = path_classify(path, 0);

        assert_eq!(Classification::SlashRooted, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 21), cr.Input);
        // assert_eq!(PoSl::new(0, 21), cr.FullPath);
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
    fn windows_path_classify_slashrooted_path() {

        use libpath::util::windows::{

            *,
        };

        {
            let path = "/dir/sub-dir/file.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::SlashRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 21), cr.Input);
            // assert_eq!(PoSl::new(0, 21), cr.FullPath);
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
            let path = "\\dir\\sub-dir\\file.ext";
            let (cl, cr) = path_classify(path, 0);

            assert_eq!(Classification::SlashRooted, cl);

            assert_ne!(ClassificationResult::empty(), cr);
            assert_eq!(PoSl::new(0, 21), cr.Input);
            // assert_eq!(PoSl::new(0, 21), cr.FullPath);
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

            assert_eq!("\\dir\\sub-dir\\file.ext", cr.Input.substring_of(path));
            assert_eq!("", cr.Prefix.substring_of(path));
            assert_eq!("\\dir\\sub-dir\\", cr.Location.substring_of(path));
            assert_eq!("", cr.Root.substring_of(path));
            assert_eq!("\\dir\\sub-dir\\", cr.Directory.substring_of(path));
            assert_eq!("file.ext", cr.Entry.substring_of(path));
            assert_eq!("file", cr.Stem.substring_of(path));
            assert_eq!(".ext", cr.Extension.substring_of(path));
        }
    }

    #[test]
    fn unix_path_classify_home_path() {

        use libpath::util::unix::{

            *,
        };

        let path = "~/dir/sub-dir/file.ext";
        let (cl, cr) = path_classify(path, 0);

        assert_eq!(Classification::HomeRooted, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 22), cr.Input);
        // assert_eq!(PoSl::new(0, 22), cr.FullPath);
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
    fn windows_path_classify_home_path() {

        use libpath::util::windows::{

            *,
        };

        let path = "~/dir/sub-dir/file.ext";
        let (cl, cr) = path_classify(path, 0);

        assert_eq!(Classification::HomeRooted, cl);

        assert_ne!(ClassificationResult::empty(), cr);
        assert_eq!(PoSl::new(0, 22), cr.Input);
        // assert_eq!(PoSl::new(0, 22), cr.FullPath);
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
}
