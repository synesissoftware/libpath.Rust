# libpath.Rust - TODO <!-- omit in toc -->


## Table of Contents <!-- omit in toc -->

- [Functional improvements](#functional-improvements)
- [Performance improvements](#performance-improvements)


## Functional improvements

* [ ] Full path support (in `libpath::util::windows`):
  * [ ] Drive-rooted paths;
  * [ ] Drive-relative paths;
  * [ ] Home-rooted paths;
  * [ ] Slash-rooted paths;
  * [ ] relative paths;
  * [ ] UNC-rooted paths;
  * [ ] UNC-incompleted paths;
  * [ ] (some of) above with:
    * [ ] Local device prefix `"\\.\"`;
    * [ ] Root local device prefix `"\\?\"`;
    * [ ] Local device prefix and UNC designator `"\\.\UNC\"`;
    * [ ] Root local device prefix and UNC designator `"\\?\UNC\"`;
    * [ ] NT path prefix `"\??\"`;
  * [ ] Device names (such as `"COM1"`);
  * [ ] Support (sadly) full flexibility in Windows paths for mixed use of `'\'` and `'/'` (though not for runs);
  * [ ] Detection of trailing space as invalid character(s);
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
* [ ] Implement path normalisation (Unix and Windows) : `to_os_normal()`, `to_asbtract_normal()`;


## Performance improvements

* \<none>


<!-- ########################### end of file ########################### -->

