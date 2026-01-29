/**
 * v1.10.1 07/10/2025
 * Author: Marco Maffei
 *
 */

// <custom
use crate::AppContext;
use crate::monitor_mod;
use crate::spinner::Spinner;
// </custom

use std::io::{self, Write};
use std::path::PathBuf;

use crossterm::{execute, terminal::{disable_raw_mode, LeaveAlternateScreen}};

use headless_chrome::{Browser, LaunchOptionsBuilder};


pub fn launch_1(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let chrome_path = Some(PathBuf::from(&app_ctx.options.chrome_path));

  let launch_options = LaunchOptionsBuilder::default()
      .headless(true)
      .sandbox(false)
      .path(chrome_path)
      .build();

  let spinner = Spinner::start("Avvio browser");

  let browser = Browser::new(launch_options?)?;
  let tab = browser.new_tab()?;
  tab.navigate_to(&app_ctx.options.url)?;
  tab.wait_until_navigated()?;

  // TODO: PROVVISORIO. LA DIALOG CHE APPARE ALL'AVVIO SUL SITO PUò ESSERE RIPRISTINATA O TOLTA. C'è UNA SOLUZIONE SU COPILOT PER GESTIRE LA PRESENZA O MENO DELLA DIALOG (CHIAMATA POP UP).
  //tab.wait_for_element(".flex.items-center.justify-center.rounded-lg.p-2.text-neutral-500.hover\\:bg-neutral-800.hover\\:text-neutral-200.focus-visible\\:outline.focus-visible\\:outline-offset-2.focus-visible\\:outline-primary")?
      //.click()?;

  tab.wait_for_element(&app_ctx.options.input_element)? // TODO: SERVE GESTIRE L'ERRORE COSI SI CAPISCE CHE NON TROVA QUESTO ELEMENTO E SI SEMPLIFICA IL DEBUG (perché ci sono altri elementi simili)
      .click()?
      .type_into(&app_ctx.options.token_quantity.to_string())?;

  //crate::dump::dump_dom_to_file(&tab, "dom_dump.html")?; // TODO:

  spinner.stop();

  let spinner = Spinner::start("Avvio Monitoraggio");

  if let Err(err) = monitor_mod::start_monitor_1(app_ctx, &tab, spinner) {
      disable_raw_mode()?;
      execute!(io::stdout(), LeaveAlternateScreen)?;

      eprintln!("\n\nErrore in start_monitor_1: {}\n", err);

      eprint!("Premi Invio per uscire…");
      io::stdout().flush()?;
      let mut _buf = String::new();
      io::stdin().read_line(&mut _buf)?;

      tab.close(true)?;

      drop(browser);
      std::process::exit(1);
  }

  Ok(())
}

pub fn launch_2(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let chrome_path = Some(PathBuf::from(&app_ctx.options.chrome_path));

  let launch_options = LaunchOptionsBuilder::default()
      .headless(true)
      .sandbox(false)
      .path(chrome_path)
      .build();

  let spinner = Spinner::start("Avvio browser");

  let browser = Browser::new(launch_options?)?;
  let tab = browser.new_tab()?;
  tab.navigate_to(&app_ctx.options.url)?;
  tab.wait_until_navigated()?;

  spinner.stop();

  let spinner = Spinner::start("Avvio Monitoraggio");

  // TODO: IN CASO DI ERRORE NON RIPORTA IL MESSAGGIO E BLOCCA IL TERMINALE IMPOSSIBILITANDO AD USCIRE DALL'APP. FORSE ESEGUE spinner.stop() IN start_monitor_2() poi succede qualcosa
  if let Err(err) = monitor_mod::start_monitor_2(app_ctx, &tab, spinner) {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        eprintln!("\n\nErrore in start_monitor_2: {}\n", err);

        eprint!("Premi Invio per uscire…");
        io::stdout().flush()?;
        let mut _buf = String::new();
        io::stdin().read_line(&mut _buf)?;

        tab.close(true)?;

        drop(browser);
        std::process::exit(1);
    }

  Ok(())
}