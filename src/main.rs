use dioxus::prelude::*;
use std::collections::HashMap;
use web_sys::window;

fn main() {
    dioxus_web::launch(app);
}

const STYLE: &str = r#"
body {
    font-family: 'Arial', sans-serif;
    background-color: #f4f4f4;
    color: #333;
    line-height: 1.6;
    margin: 0;
    padding-top: 60px; /* Adjusted to prevent content from being hidden behind the fixed navbar */
}

.navbar {
    background-color: #333;
    color: white;
    position: fixed;
    top: 0;
    width: 100%;
    height: 50px;
    display: flex;
    justify-content: start; /* Align items to the start of the navbar */
    align-items: center;
    padding: 0 20px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.navbar div {
    display: flex;
    align-items: center;
}

.navbar a, .navbar button {
    display: inline-flex; /* Use inline-flex to align text centrally */
    align-items: center;
    justify-content: center;
    color: white;
    text-decoration: none;
    margin: 0 10px;
    padding: 5px 10px;
    border-radius: 5px;
    background-color: #333;
    border: none;
    cursor: pointer;
    font-size: 16px;
    transition: background-color 0.3s; /* Smooth transition for hover effect */
}

.navbar a:hover, .navbar button:hover {
    background-color: #0056b3;
    color: white;
}


.navbar a, .navbar a:hover {
    height: 20px;
}

.container {
    width: 90%;
    max-width: 1200px;
    margin: auto;
    overflow: hidden;
    padding: 20px;
}

h1, h2 {
    color: #0056b3;
}

p {
    margin: 15px 0;
}

a {
    color: #007bff;
    text-decoration: none;
}

a:hover {
    color: #0056b3;
}

.section {
    padding: 20px 0;
    border-bottom: 1px solid #eaeaea;
}

.feature-list ul {
    list-style-type: disc;
    padding-left: 20px;
}

.feature-list ul ul {
    list-style-type: circle;
    padding-left: 20px;
}

.feature-list li {
    margin-bottom: 10px;
}

.playground label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
}

.playground input[type='text'], .playground textarea {
    width: 100%;
    padding: 10px;
    margin-bottom: 10px;
    border: 1px solid #ddd;
}

.playground button {
    display: inline-block;
    background: #28a745;
    color: white;
    padding: 10px 15px;
    border: none;
    cursor: pointer;
}

.playground button:hover {
    background: #218838;
}

#output {
    background: #eee;
    padding: 10px;
    border: 1px solid #ddd;
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

    let scroll_to_section = |id: String| {
        if let Some(window) = window() {
            if let Some(element) = window.document().unwrap().get_element_by_id(&id) {
                element.scroll_into_view();
            }
        }
    };

    cx.render(rsx!(
        style {{ STYLE }}
        title { "SSD - Simple Service & Data Description" }
        nav { class: "navbar",
            div {
                button { onclick: move |_| scroll_to_section("history".to_string()), "History" }
                button { onclick: move |_| scroll_to_section("features".to_string()), "Features" }
                button { onclick: move |_| scroll_to_section("playground".to_string()), "Playground" }
            }
            div {
                a { href: "https://github.com/ssd-codegen/ssd", "GitHub" }
            }
        }
        div { class: "container",
            h1 { "SSD - Simple Service & Data Description" }
            p { "First and foremost it's supposed to be data format for describing data structures and services. Additionally it provides tooling to work with the aforementioned format, which simplifies writing custom code generators." }

            h2 { "Think this page is ugly? Help make it look more beautiful by submitting a PR at " a { href: "https://github.com/ssd-codegen/ssd", "https://github.com/ssd-codegen/ssd" } }

            section { id: "history", class: "history",
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

            section { id: "features", class: "feature-list",
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

            section { id: "playground", class: "playground",
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
