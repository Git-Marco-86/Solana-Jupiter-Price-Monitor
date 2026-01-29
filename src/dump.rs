/**
 * v1.0.0 27/09/2025
 * Author: Marco Maffei
 *
 */

use std::path::Path;
use headless_chrome::Tab;

/// Dump del DOM completo (doctype + documentElement.outerHTML) su file
#[allow(dead_code)]
pub fn dump_dom_to_file(tab: &Tab, out_path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let js = r#"(() => {
        const d = document;
        const doctype = d.doctype ? new XMLSerializer().serializeToString(d.doctype) : '';
        return doctype + d.documentElement.outerHTML;
    })()"#;

    let eval_res = tab.evaluate(js, false)?;
    let html = eval_res
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let out = out_path.as_ref();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(out, html)?;
    Ok(())
}


// TODO:
  // stampa l'elemento
  /*let outer_html_js = "function() { return this.outerHTML; }";
  let html_result = input_price.call_js_fn(outer_html_js, vec![], true)?;
  if let Some(value) = html_result.value {
      if let Some(outer_html) = value.as_str() {
          println!("HTML dell'elemento: {}\r", outer_html);
      } else {
          eprintln!("Il valore ottenuto non è una stringa: {:?}\r", value);
      }
  } else {
      eprintln!("Impossibile ottenere l'HTML dall'elemento.\r");
  }*/

  // tutti i valori
  /*
  match input_price.get_attributes() {
Ok(Some(attrs_vec)) => {
  println!("Attributi dell'elemento: {:?}", attrs_vec);
  // Il resto del codice per cercare "value"
  if let Some(idx) = attrs_vec.iter().position(|attr| attr == "value") {
      if idx + 1 < attrs_vec.len() {
          let value = &attrs_vec[idx + 1];
          println!("Il valore dell'input è: {}", value);
      } else {
          eprintln!("Chiave 'value' trovata ma nessun valore associato.");
      }
  } else {
      eprintln!("L'attributo 'value' non è presente.");
  }
}
Ok(None) => eprintln!("Nessun attributo trovato."),
Err(e) => eprintln!("Errore nel recuperare gli attributi: {}", e),
}
  */


/* TODO: FUNZIONE CHE MI HA AIUTATO A CAPIRE L'ERRORE PER PRENDERE IL TESTO IN <SPAN>
  let expr = format!(
    "document.querySelector('{}').outerHTML",
    &app_ctx.options.output_element
);
let res = tab.evaluate(&expr, false)?;  // false = non è async
let html = res
    .value
    .and_then(|v| v.as_str().map(String::from))
    .unwrap_or_default();

    */




  /* // TODO: COME SOPRA

  let selector = &app_ctx.options.output_element;

// 1) aspetta che innerText non sia più vuoto
let wait_iife = format!(
    r#"(() => {{
        const el = document.querySelector("{sel}");
        return !!(el && el.innerText.trim().length > 0);
    }})()"#,
    sel = selector
);
tab.evaluate(&wait_iife, false)?; // false perché non è async

// 2) estrai il testo
let get_iife = format!(
    r#"(() =>
        document.querySelector("{sel}").innerText.trim()
    )()"#,
    sel = selector
);
let res = tab.evaluate(&get_iife, false)?;
let html = res
    .value
    .and_then(|v| v.as_str().map(str::to_string))
    .unwrap_or_default();


   */