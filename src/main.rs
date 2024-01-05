use dioxus::prelude::*;
use std::collections::HashMap;

fn main() {
    dioxus_web::launch(app);
}

const STYLE: &str = r#"
            .container {
                font-family: Arial, sans-serif;
                margin: 20px;
                padding: 10px;
            }
            h1, h2 {
                color: #333;
            }
            p, label {
                font-size: 16px;
            }
            .history {
                max-width: 80%;
            }
            .feature-list, .playground {
                margin-top: 20px;
            }
            .playground textarea, .playground input[type='text'] {
                width: 100%;
                margin-top: 5px;
                margin-bottom: 10px;
                padding: 8px;
                font-size: 14px;
                box-sizing: border-box;
                border: 1px solid #ddd;
                border-radius: 4px;
            }
            .playground button {
                margin-top: 10px;
                padding: 10px 15px;
                background-color: #28a745;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
            }
            .playground button:hover {
                background-color: #218838;
            }
            #output {
                margin-top: 20px;
                padding: 10px;
                border: 1px solid #ddd;
                background-color: #f8f9fa;
            }
        "#;

fn app(cx: Scope) -> Element {
    let result = use_state(cx, || Option::<String>::None);
    let namespace = use_state(&cx, || "my::namespace".to_string());
    let data = use_state(&cx, || include_str!("../data/test.svc").to_string());
    let rhai_code = use_state(&cx, || {
        include_str!("../example-generators/cpp-like.rhai").to_string()
    });
    let type_mappings = use_state(&cx, || {
        include_str!("../example-generators/cpp-like.tym").to_string()
    });

    let debug_mode = use_state(&cx, || false);

    cx.render(rsx!(
        style {{ STYLE }}
        div { class: "container",
            h1 { "SSD - Simple Service & Data Description" }
            p { "First and foremost it's supposed to be data format for describing data structures and services. Additionally it provides tooling to work with the aforementioned format, which simplifies writing custom code generators." }

            h2 { "Think this page is ugly? Help make it look more beautiful by submitting a PR at " a { href: "https://github.com/ssd-codegen/ssd", "https://github.com/ssd-codegen/ssd" } }

            section { class: "history",
                h2 { "History - Why did I make this?" }

                p { r#"In one of the companies I was working we programmed in Delphi and we had a bunch of structs that needed to
be serialized. In order to serialize stuff in Delphi you need build the objects manually,
which was really error prone and annoying to work with. So I wrote a small (closed source)
code generator using "#r a {href: "https://dlang.org/", "D" } ", " a { href: "https://sdlang.org/", "SDLang" } " and a template engine." }

p { r#"After a few months it was clear, that the template engine made certain things way harder to maintain and reason
about. Which is why I decided to rewrite the whole thing in C#. This time I used a custom parser to allow for more
streamlined data files and I built the generator in a way, that the actual code generation would be done through
C# DLLs. This way you could still use a template engine if you want by embedding it into a DLL and using that."#r }

p { r#"I was even allowed to open source everything except the custom code generation plugin we used internally.
The source code can be found here: "#r a { href: "https://github.com/hardliner66/codegenerator", "https://github.com/hardliner66/codegenerator" } }

p { r#"I was still not really satisfied, as using the codegen in a cross platform way was still tricky and require mono.
After some time I was starting to use rust more and found out about webassembly,
which is what motivated me to start a third attempt. This time the goal was to allow plugins to be written in wasm,
to allow people to write their generators in whatever language they're comfortable with."#r }

p { "I called the project SSDCG first, which stood for " b { "S" } "imple " b { "S" } "ervice and " b { "D" } "ata description format and " b { "C" }
"ode " b { "G" } r#"enerator. But the name was kinda hard to remember and the focus was always more on the unified data description
language, with the code generation being something that was the main use case for that language."#r }

p { r#"The project has already outgrown it's initial goal by supporting not only WASM,
but also Rhai (script language) and three different template engines, that can all work with the same unified data
model. Once your model is written, you can choose whatever technology fits your need the best to generate whatever
you want out from the model."#r }

p { r#"The data format also evolved quite a bit from the older versions and supports describing DataTypes, Enums, Imports,
Services with functions and events, as well as custom attributes on all of these to allow for even more customization.
It's modelled to be similar to a simplified rust, because I personally like the syntax quite a bit and it was a
natural choice, as the whole thing is written in rust itself as well."#r }
            }

            section { class: "feature-list",
                h2 { "Features" }
                a { href: "https://github.com/ssd-codegen/ssd#features", "Full Feature List" }
                ul {
                    li { "Custom Data and Service Description Language"
                        ul {
                            li { "Imports" }
                            li { "DataTypes" }
                            li { "Enums" }
                            li { "Services" }
                            li { "Custom Attributes"
                                ul {
                                    li { "These can be used to implement custom features that are missing from the language" }
                                }
                            }
                            li { "Lists"
                                ul {
                                    li { "Fixed Size (property: 5 of u8)" }
                                    li { "Dynamic Size (property: list of u8)" }
                                }
                            }
                        }
                    }
                    li { "Scripting Languages"
                        ul {
                            li { "Rhai" }
                            li { "Python through PyO3" }
                        }
                    }
                    li { "Template Engines"
                        ul {
                            li { "Handlebars" }
                            li { "Tera" }
                        }
                    }
                    li { "Wasm through " a { href: "https://extism.org/", "extism" } }
                    li { "Using raw data (JSON, Yaml, Toml, Rsn) instead of predefined ssd format"
                        ul {
                            li { "This allows the same tool to be used, even when working with data from somewhere else" }
                        }
                    }
                }
            }

            section { class: "playground",
                h2 { "Playground" }
                form {
                    label { "Namespace: " }
                    input {
                        r#type: "text",
                        value: "{namespace}",
                        oninput: |e| namespace.set(e.value.clone()),
                    }

                    label { "Custom Data Format: " }
                    textarea {
                        rows: "25",
                        value: "{data}",
                        oninput: |e| data.set(e.value.clone()),
                    }

                    label { "Rhai Code: " }
                    textarea {
                        rows: "25",
                        value: "{rhai_code}",
                        oninput: |e| rhai_code.set(e.value.clone()),
                    }

                    label { "Type Mappings: " }
                    textarea {
                        rows: "5",
                        value: "{type_mappings}",
                        oninput: |e| type_mappings.set(e.value.clone()),
                    }

                    button {
                        prevent_default: "onclick",
                        onclick: |_| {
                            let data = data.get().trim();
                            let namespace = namespace.get().trim();
                            let typemap = type_mappings.get().trim();
                            let rhai = rhai_code.get().trim();
                            match ssd::generate_web(HashMap::default(), &namespace, &rhai, &typemap, &data, *debug_mode.get()) {
                                Ok(r) => result.set(Some(dbg!(r))),
                                Err(e) => result.set(Some(format!("{e}"))),
                            }
                        },
                        "Run"
                    }
                    // label {
                    //     "Debug: "
                    //     input {
                    //         r#type: "checkbox",
                    //         checked: "{debug_mode}",
                    //         onchange: |e| debug_mode.set(e.value.parse().unwrap_or(false)),
                    //     }
                    // }

                    // Placeholder for the output box, initially hidden
                    div { id: "output", style: if result.is_none() { "display: none;" } else { "" }, pre { result.get().clone().unwrap_or_default()} }
                }
            }
        }
    ))
}
