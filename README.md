# apicheck

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT)
[![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

## Overview

apicheck is a tool for extracting a description of the public API of a file (or crate) into JSON format.
It is useful, for example, for checking if the public API of a library did not change between two versions,
resulting in breaking changes for all crates that are using your library.

It can be used to analyze your API, or check if semver is correctly applied (no API break in minor version update).

`apicheck` takes a file as input and will parse it recursively. For crates, run it on the `src/lib.rs` file.
It will describe all public items (ignoring symbols restricted to crate or non-public), giving all information
it knows about the items.

There are several tools:

* `apicheck`: the main executable
* `apicheck_plugin`: a compiler plugin to extract API

## Building

`apicheck` uses nightly features, so you must use the nightly compiler.

## Calling apicheck

Since `apicheck` uses features from the rust compiler, it is linked to shared libraries (especially `libstd-xxx.so`,

`libsyntax_pos-xxx.so` and `librustc_errors-xxx.so`), which may not be in your PATH.

Some solutions:

Add the path to these libraries to the loader path:

```shell
export LD_LIBRARY_PATH=`rustc --print sysroot`/lib/rustlib/x86_64-unknown-linux-gnu/lib
apicheck tests/simple.rs
```

Or, use `cargo run` as a prefix to your command:

```shell
cargo run apicheck -- tests/simple.rs
```

## Example

We'll use the following code as an example file ([tests/01.rs](tests/01.rs)):

```rust
pub fn visible_function(i: u32) -> u32 { i + 1 }

pub fn fun01() { }
```

To extract the public API, run:

```shell
cargo run apicheck -- -d ./tests/01.rs | tail -1 > 01.json
```

Output is a single-line, compact JSON file.

To pretty-print it, your best friend is [jq](https://github.com/stedolan/jq)!
See https://stedolan.github.io/jq/manual/#Basicfilters for more help

```shell
$ cargo run apicheck -- -d ./tests/01.rs |tail -1 |jq
```
```json
{
  "modules": [
    {
      "path": "./tests/01.rs",
      "items": [
        {
          "type": "function",
          "name": "visible_function",
          "inputs": [
            {
              "type": "u32",
              "name": "i"
            }
          ],
          "output": "u32",
          "variadic": false,
          "unsafety": "normal",
          "constness": "",
          "abi": "Rust",
          "generics": "",
          "where": "",
          "visibility": "public",
          "attrs": []
        },
        {
          "type": "function",
          "name": "fun01",
          "inputs": [],
          "output": "",
          "variadic": false,
          "unsafety": "normal",
          "constness": "",
          "abi": "Rust",
          "generics": "",
          "where": "",
          "visibility": "public",
          "attrs": []
        }
      ]
    }
  ]
}
```

Now, suppose we change the return type of `visible_function` ([tests/02.rs](tests/02.rs))::

```rust
pub fn visible_function(i: u32) -> usize { (i + 1) as usize }

pub fn fun01() { }
```

Extract the API again, and the arguments of `visible_function` will now differ:

```json
          "type": "function",
          "name": "visible_function",
          "inputs": [
            {
              "type": "u32",
              "name": "i"
            }
          ],
          "output": "usize",
                    ^^^^^^^
```

To check for differences, we can use a JSON diff tool, or even `diff` (after pretty-printing files):

```shell
$ cargo run -- -d ./tests/01.rs |tail -1 | jq . > 01.json
$ cargo run -- -d ./tests/02.rs |tail -1 | jq . > 02.json
```
```diff
diff -u 01.json 02.json 
--- 01.json     2018-05-30 08:26:39.234748749 +0200
+++ 02.json     2018-05-30 08:26:53.433847330 +0200
@@ -1,7 +1,7 @@
 {
   "modules": [
     {
-      "path": "./tests/01.rs",
+      "path": "./tests/02.rs",
       "items": [
         {
           "type": "function",
@@ -12,7 +12,7 @@
               "name": "i"
             }
           ],
-          "output": "u32",
+          "output": "usize",
           "variadic": false,
           "unsafety": "normal",
           "constness": "",
```

## Tips

### Sorting items

It is often useful to sort items, else moving code will result in changes in the JSON file.
To sort the file entries using `jq`, run

```shell
jq '(.. | arrays) |= sort' 02.json

```
However, this may have side-effects, like changing the order of function arguments.

## TODO

Help needed!

- [ ] write an `apidiff` tool to semantically diff JSONs
- [ ] write a cargo subcommand

## Limitations

- Macros are not expanded, and are not analyzed as part of the API. As `apicheck` relies on the rust syntax parser, it
  macros are not interpreted, thus it is currently not possible to support them.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

