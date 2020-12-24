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
* `apidiff`: a tool to compare to JSON files extracted by `apicheck`

## Building

`apicheck` uses nightly features, so you must use the nightly compiler.

```
export CFG_RELEASE_CHANNEL=nightly
export CFG_RELEASE=nightly
cargo +nightly build --all
```

## Calling apicheck

```shell
apicheck tests/simple.rs
```

Or, use `cargo run` as a prefix to your command:

```shell
cargo run --bin apicheck -- tests/simple.rs
```

## Example

We'll use the following code as an example file ([tests/01.rs](tests/01.rs)):

```rust
pub fn visible_function(i: u32) -> u32 { i + 1 }

pub fn fun01() { }
```

To extract the public API, run:

```shell
cargo run apicheck -- ./tests/01.rs > 01.json
```

Output is a single-line, compact JSON file.

To pretty-print it, your best friend is [jq](https://github.com/stedolan/jq)!
See https://stedolan.github.io/jq/manual/#Basicfilters for more help

```shell
$ cargo run apicheck -- ./tests/01.rs |jq
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
$ cargo run -- ./tests/01.rs jq . > 01.json
$ cargo run -- ./tests/02.rs jq . > 02.json
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

The `apidiff` tool is still experimental, but it can check differences and exit with a non-zero code if found::

```shell
$ RUST_LOG=apidiff=debug ./target/debug/apidiff -v -p 3 ./assets/01.json ./assets/02.json
DEBUG 2019-12-11T08:56:39Z: apidiff: ***
 INFO 2019-12-11T08:56:39Z: apidiff: Item 'visible_function': property 'output' has changed from 'u32' to 'usize'
DEBUG 2019-12-11T08:56:39Z: apidiff: ***
DEBUG 2019-12-11T08:56:39Z: apidiff: ***
Summary:
    Modules added: 0
    Modules removed: 0
    Modules changed: 1
    Items added: 0
    Items removed: 0
    Items changed: 1
```

## Tips

### Sorting items

It is often useful to sort items, else moving code will result in changes in the JSON file.
To sort the file entries using `jq`, run

```shell
jq '(.. | arrays) |= sort' 02.json

```
However, this may have side-effects, like changing the order of function arguments.

### Modules stats

`jq` can be used to show stats:

```shell
$ jq '[.modules[].items[].type] | sort[]' /tmp/libapicheck.rs | uniq -c
      3 "enum"
      4 "function"
      4 "impl"
      1 "mod"
      1 "struct"
      1 "usetree"
```

## TODO

Help needed!

- [x] write an `apidiff` tool to semantically diff JSONs (in progress)
  - [ ] show differences
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

