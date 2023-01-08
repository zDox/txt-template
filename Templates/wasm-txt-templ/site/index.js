import("./node_modules/wasm-txt-templ/wasm_txt_templ.js").then(module => {
  try {
    // Load a template by parsing it.
    const input = "Hallo {name}";
    document.getElementById("input").innerHTML = input;
    const template = module.Template.parse(input);

    // Get a draft of the content table required by the template
    // and fill in the single missing value.
    let content_map = JSON.parse(template.draft());
    content_map[0][1] = "Paul";

    // Use the content map to fill out the missing values
    // in the loaded template.
    const output = template.fill_out(JSON.stringify(content_map));
    document.getElementById("output").innerHTML = output;
  } catch(e) {
    console.error(e);
  }
});
