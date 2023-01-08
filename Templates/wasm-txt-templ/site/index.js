import("./node_modules/wasm-txt-templ/wasm_txt_templ.js").then(module => {
  try {
    // Load a template by parsing it.
    let template = module.Template.parse("Hello {name}");

    // JSON representation of values used to  fill out the missing entries
    // in the template. 
    const content_map = '[[["name", "Key"], "Paul"]]';

    // TODO: Provide a content map containing all emtpy entires from a loaded template
    // e.g.:
    // let content_map = template.get_draft();  // This would return '[[["name", "Key"], ""]]' for the above example

    // Use the content map to fill out the missing values
    // in the loaded template.
    let filled_out = template.fill_out(content_map);
    console.log(filled_out);
  } catch(e) {
    console.error(e);
  }
});
