import("./node_modules/wasm-txt-templ/wasm_txt_templ.js").then(module => {
  try {
    // Load a template by parsing it.
    const input = "Hallo {name}";
    document.getElementById("input").innerHTML = input;
    const template = module.Template.parse(input);

    // Get a draft of the content table required by the template
    // and fill in the single missing value.
    var content_map = template.draft();
    content_map.set(key = ["name", "Key"], "Paul");

    // Use the content map to fill out the missing values
    // in the loaded template.
    const output = template.fill_out(content_map);
    document.getElementById("output").innerHTML = output;
  } catch(e) {
    console.error(e);
  }
});
