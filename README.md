# SSD - Simple Service Description

## ATTENTION: BREAKING CHANGE!
I changed the syntax from `handles` to `fn` and the field from `handlers` to `functions`.

## Features
- [x] Custom description language (basics are done, but some things are still missing)
- [ ] Auto format
- [ ] Basic sanity checks
- [x] Run RHAI scripts to generate output

## Future Features
- [ ] Run WASM plugins to generate output

You can check out the file:
- [data/test.svc](./data/test.svc) to see what the description language looks like.
- [generators/cpp-like.rhai](./generators/cpp-like.rhai) to see what a generator could look like.
- [generators/cpp-like.rhai.tym](./generators/cpp-like.tym) to see what a typemapping file looks like.
- [generators/simple.hbs](./generators/simple.hbs) to see what a simple handlebars template looks like.
- [generators/simple.lqd](./generators/simple.lqd) to see what a simple liquid template looks like.
- [generators/simple.tera](./generators/simple.tera) to see what a simple tera template looks like.

## Install
```shell
cargo install --locked ssdcg
```

## Usage
### General
```shell
➜ ssd help
Simple Service Description & Code Generator

Usage: ssd [COMMAND]

Commands:
  debug          Print debug representation of the parsed file
  generate       Generate source code
  rhai-metadata  Print script engine metadata (function definitions, etc.) as json
  help           Print this message or the help of the given subcommand(s)

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
  liquid      Use a liquid based templates. https://shopify.github.io/liquid/
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
          
          If a file with the same name as the script file, but with the extension tym, it will be used automatically. e.g.: If there is a file
          `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get used automatically.

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help information (use `-h` for a summary)
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
          
          If a file with the same name as the script file, but with the extension tym, it will be used automatically. e.g.: If there is a file
          `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get used automatically.

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
  <TEMPLATE_DIR>   Glob path for where to search for templates
  <TEMPLATE_NAME>  The template to use to generate the file
  <FILE>           which file to use

Options:
      --typemap <TYPEMAP>  A file containing type mappings
  -o, --out <OUT>          The file which should get written with the output from the generator
  -h, --help               Print help
```

#### Liquid
Alias: `ssd generate lqd`

```shell
➜ ssd generate liquid --help
Use a liquid based templates. https://shopify.github.io/liquid/

Usage: ssd generate liquid [OPTIONS] <TEMPLATE> <FILE>

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
          
          If a file with the same name as the script file, but with the extension tym, it will be used automatically. e.g.: If there is a file
          `/generator/script.rhai` and a corresponding `/generator/script.tym`, it will get used automatically.

  -o, --out <OUT>
          The file which should get written with the output from the generator

  -h, --help
          Print help (see a summary with '-h')
```


To test it out you can use the following command:
```rust
ssd generate rhai generators/cpp-like.rhai data/test.svc
```
