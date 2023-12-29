# SSD - Simple Service & Data Description

## About - What is SSD?

First and foremost it's supposed to be data format for describing data structures and services.
Additionally it provides tooling to work with the aforementioned format, which simplifies writing custom code generators.

## History - Why did I make this?

In one of the companies I was working we programmed in Delphi and we had a bunch of structs that needed to
be serialized. In order to serialize stuff in Delphi you need build the objects manually,
which was really error prone and annoying to work with. So I wrote a small (closed source)
code generator using [D](https://dlang.org/), [SDLang](https://sdlang.org/) and a template engine.

After a few months it was clear, that the template engine made certain things way harder to maintain and reason
about. Which is why I decided to rewrite the whole thing in C#. Thio time I used a custom parser to allow for more
streamlined data files and I built the generator in a way, that the actual code generation would be done through
C# DLLs. This way you could still use a template engine if you want by embedding it into a DLL and using that.

I was even allowed to open source everything except the custom code generation plugin we used internally.
The source code can be found here: https://github.com/hardliner66/codegenerator

I was still not really satisfied, as using the codegen in a cross platform way was still tricky and require mono.
After some time I was starting to use rust more and found out about webassembly,
which is what motivated me to start a third attempt. This time the goal was to allow plugins to be written in wasm,
to allow people to write their generators in whatever language they're comfortable with.

I called the project SSDCG first, which stood for **S**imple **S**ervice and **D**ata description format and **C**ode
**G**enerator. But the name was kinda hard to remember and the focus was always more on the unified data description
language, with the code generation being something that was the main use case for that language.

The project has already outgrown it's initial goal by supporting not only WASM,
but also Rhai (script language) and three different template engines, that can all work with the same unified data
model. Once your model is written, you can choose whatever technology fits your need the best to generate whatever
you want out from the model.

The data format also evolved quite a bit from the older versions and supports describing DataTypes, Enums, Imports,
Services with functions and events, as well as custom attributes on all of these to allow for even more customization.
It's modelled to be similar to a simplified rust, because I personally like the syntax quite a bit and it was a
natural choice, as the whole thing is written in rust itself as well.

## ATTENTION: BREAKING CHANGES!
As long as the crate version is below 1.0.0, breaking changes are to be expected.

Breaking changes so far:
| Version | Change                                                                                                                 |
| ------- | ---------------------------------------------------------------------------------------------------------------------- |
| 0.8.0   | I changed the syntax from `handles` to `fn` and the field from `handlers` to `functions`                               |
| 0.9.0   | Rename crate to ssd                                                                                                    |
| 0.10.0  | Move AST to separate crate for use in wasm plugins                                                                     |
| 0.11.0  | Restrict places where comments can appear. This simplifies auto-formatting.                                            |
| 0.12.0  | Rename SsdcFile to SsdFile so it matches the project name                                                              |
| 0.13.0  | Doc-Comments are now officially part of the exposed data.                                                              |
| 0.14.0  | Remove liquid templates to simplify the code and remove code duplication. Tera seems close enough anyway.              |
| 0.15.0  | Put Ron behind a feature gate, as I already had some problems with it before and provide rsn (similar format) instead. |
| 0.16.0  | Change representation of properties from indexmap to vec of tuple.                                                     |
| 0.17.0  | Renamed `SsdFile` to `SsdModule`. Removed `wasm` and `tera` from the default features.                                 |

## Features
* [x] Custom description language (basics are done, but some things are still missing)
  * [x] Imports
  * [x] DataTypes
  * [x] Enums
  * [x] Services
  * [x] Custom Attributes
    * These can be used to implement custom features that are missing from the language
    * Some features will get added later, others will always rely on attributes, because they aren't generic enough
  * [x] Lists
    * Fixed Size (`property: 5 of u8`)
    * Dynamic Size (`property: list of u8`)
  * [ ] Generics
* [x] Auto format
* Script Languages
   * [x] [Rhai](https://rhai.rs/)
   * [x] Python through PyO3
* Template Engines
   * [x] [Handlebars](https://handlebarsjs.com/)
   * [x] [Tera](https://keats.github.io/tera/)
* [x] Wasm (through [extism](https://extism.org/))
* [x] Data Export for use with other tools (JSON, Yaml, Toml)
* [x] Use raw data (JSON, Yaml, Toml, Rsn) instead of predefined ssd format
   * This allows the same tool to be used, even when working with data from somewhere else
* [ ] Basic sanity checks

### Cargo Features
- `default` is `wasm`, `tera`, `handlebars`
- `tera` enables support for tera templates
- `handlebars` enables support for handlebars templates
- `wasm` enables support for wasm plugins
- `ron` enables support for `ron`
- `all` enables everything

## Data Specification
It's mostly "what you see is what you get", as seen here:
- [data/test.svc](./data/test.svc) to see what the description language looks like.

Only restriction for now, is that auto-format will always put comments before the element right after. This means the following
```
data Test {
    a: i32, /// test
    b: i32,
}
```

will get formatted to:
```
data Test {
    a: i32,
    /// test
    b: i32,
}
```

## Test it out

To test it out, install the command, clone the repository and use the following command:
```rust
ssd generate rhai example-generators/cpp-like.rhai data/test.svc
```

## Examples

You can check out the files:
- [example-generators/cpp-like.rhai](./example-generators/cpp-like.rhai) to see what a generator could look like.
- [example-generators/cpp-like.rhai.tym](./example-generators/cpp-like.tym) to see what a typemapping file looks like.
- [example-generators/simple.hbs](./example-generators/simple.hbs) to see what a simple handlebars template looks like.
- [example-generators/simple.tera](./example-generators/simple.tera) to see what a simple tera template looks like.
- [example-generators/wasm-example/README.md](./example-generators/wasm-example/README.md) to see what a simple generator in rust (wasm) looks like.

## Install

Either install through cargo:
```shell
cargo install --locked ssd
```

or download pre-built package from [releases page](https://github.com/ssd-codegen/ssd/releases/latest).

## Usage
### General
```shell
➜ ssd help
Simple Service Description

Usage: ssd [COMMAND]

Commands:
  debug     Print debug representation of the parsed file
  pretty    Pretty print the parsed file
  generate  Generate source code
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Generate
```shell
➜ ssd generate help
Generate source code

Usage: ssd generate <COMMAND>

Commands:
  rhai        Use a rhai based generator
  handlebars  Use a handlebars based template. https://handlebarsjs.com/
  tera        Use a tera based template. https://tera.netlify.app/
  wasm        Use a wasm based generator
  data        Output as serialized data for external use
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### Rhai
```shell
➜ ssd generate rhai --help
Use a rhai based generator

Usage: ssd generate rhai [OPTIONS] <SCRIPT> <FILE>

Arguments:
  <SCRIPT>
          The script to use to generate the file

  <FILE>
          which file to use

Options:
  -d, --debug
          Enables debug mode (print and debug function in the script)

      --no-map
          do not use type mappings

      --typemap <TYPEMAP>
          A file containing type mappings.

          If a file with the same name as the script file, but with the extension tym, it will be used automatically.
          e.g.: If there is a file `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get
          used automatically.

  -r, --raw
          use raw data file as input instead of the ssd data format

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help (see a summary with '-h')
```

#### Handlebars
Alias: `ssd generate hbs`

```shell
➜ ssd generate handlebars --help
Use a handlebars based template. https://handlebarsjs.com/

Usage: ssd generate handlebars [OPTIONS] <TEMPLATE> <FILE>

Arguments:
  <TEMPLATE>
          The template to use to generate the file

  <FILE>
          which file to use

Options:
      --no-map
          do not use type mappings

      --typemap <TYPEMAP>
          A file containing type mappings.

          If a file with the same name as the script file, but with the extension tym, it will be used automatically.
          e.g.: If there is a file `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get
          used automatically.

  -r, --raw
          use raw data file as input instead of the scd data format

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help (see a summary with '-h')
```

#### Tera
```shell
➜ ssd generate tera --help
Use a tera based template. https://tera.netlify.app/

Usage: ssd generate tera [OPTIONS] <TEMPLATE_DIR> <TEMPLATE_NAME> <FILE>

Arguments:
  <TEMPLATE_DIR>
          Glob path for where to search for templates

  <TEMPLATE_NAME>
          The template to use to generate the file

  <FILE>
          which file to use

Options:
      --no-map
          do not use type mappings

      --typemap <TYPEMAP>
          A file containing type mappings.

          If a file with the same name as the script file, but with the extension tym, it will be used automatically.
          e.g.: If there is a file `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get
          used automatically.

  -r, --raw
          use raw data file as input instead of the scd data format

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help (see a summary with '-h')
```

#### Wasm

```shell
➜ ssd generate wasm --help
Use a wasm based generator

Usage: ssd generate wasm [OPTIONS] <WASM> <FILE>

Arguments:
  <WASM>
          The wasm plugin to use to generate the file

  <FILE>
          which file to use

Options:
      --no-map
          do not use type mappings

      --typemap <TYPEMAP>
          A file containing type mappings.

          If a file with the same name as the script file, but with the extension tym, it will be used automatically.
          e.g.: If there is a file `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get
          used automatically.

  -r, --raw
          use raw data file as input instead of the scd data format

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help (see a summary with '-h')
```

## Python / PyO3
Install through pip:
```sh
pip3 install py_ssd
```

### Usage
```py
>>> import py_ssd
>>> model = py_ssd.parse_file(".", "./data/test.svc")
[src/parser.rs:509] key = "Rect"

# the namespace is generated from the file path (second parameter),
# with the base path removed (first parameter)
>>> model['namespace']
{'components': ['data', 'test']}

>>> model['data_types'].keys()
dict_keys(['Rect'])

>>> model['data_types']['Rect']['properties']['x']
{'typ': {'components': ['i32']}, 'attributes': [{'name': {'components': ['test']}, 'parameters': []}], 'comments': []}
```