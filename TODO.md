# libpath.Rust - TODO <!-- omit in toc -->


## Table of Contents <!-- omit in toc -->

- [Functional improvements](#functional-improvements)
- [Performance improvements](#performance-improvements)


## Functional improvements

* [ ] Full UNC support (in `libpath::util::windows`);
* [ ] Rename `ClassificationResult` to `PathDescriptor`;
* [x] Rename `ClassificationResult#Entry` to `#EntryName`;
* [ ] Remove `ClassificationResult#FirstInvalid` and use in function calls;
* [x] Correct handling of entry-names with trailing `'.'` character(s);
* [ ] Add many and varied test cases with invalid characters / names;
* [ ] Add trait `Path` that provides access to elements (as `&str`, etc.);
* [ ] Implement `IGNORE_SLASH_RUNS`;
* [ ] Implement `IGNORE_INVALID_CHARS`;
* [ ] Implement `RECOGNISE_TILDE_HOME`;
* [ ] Implement `IGNORE_INVALID_CHARS_IN_LONG_PATH`;


## Performance improvements

* \<none>


<!-- ########################### end of file ########################### -->

