/**
 * v3.1.0 28/09/2025
 * Author: Marco Maffei
 *
 */

use crate::CONFIG_RUN;
use crate::config::MENU_START_ROW;
use crate::logger::Logger;
use crate::options::Options;
use crate::tokens::TOKENS;
use crate::generic_mod;
use crate::AppContext;
use crate::audio_assets::AudioAssets;
use crate::gui::{draw_full_menu, redraw, redraw_selection, print_header, print_commands, prompt_and_read};

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor::{self, Show, Hide},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    queue,
    terminal::{self, ClearType, EnterAlternateScreen},
};

// TODO: INCAPSULARE DA QUALCHE PARTE
fn set_menu_items_monitoring(app_ctx: &mut AppContext, lang: &String) -> (Vec<String>, &'static str) {
  
  let token_info = TOKENS.get(&app_ctx.options.token_id).unwrap(); // TODO: MANCA DEFAULT
  let token_name = &token_info.symbol;

  let menu_items: Vec<String> = vec![
      format!("Percorso Chrome o Chromium: {}", app_ctx.options.chrome_path),
      format!("Visualizzazione: {}", app_ctx.options.view_type.label(&lang)),
      format!("Intervallo di monitoraggio (s): {}", app_ctx.options.monitoring_interval_secs),
      format!("Abilita logs: {}", if app_ctx.options.logs_enabled { "Si" } else { "No" }),
      format!("Interrompi a prezzo minimo o massimo: {}", if app_ctx.options.stop_when_price_reached { "Si" } else { "No" }),
      format!("Token: {}", token_name),
      format!("Quantità: {}", app_ctx.options.token_quantity),
      format!("Prezzo minimo: {}", app_ctx.options.min_price),
      format!("Prezzo massimo: {}", app_ctx.options.max_price),
      format!("Ribasso %: {}", app_ctx.options.down_price_perc),
      format!("Rialzo %: {}", app_ctx.options.up_price_perc),
      String::from("INDIETRO"),
  ];

  (menu_items, &token_name)
}

// TODO: INCAPSULARE DA QUALCHE PARTE
#[allow(unused_variables)]
fn set_menu_items_options(app_ctx: &mut AppContext, lang: &String) -> Vec<String> {

  let menu_items: Vec<String> = vec![
      format!("Abilita suono: {}", if app_ctx.options.sound_enabled { "Si" } else { "No" }),
      format!("Audio prezzo minimo raggiunto: {}", app_ctx.options.audio_min_price_file_name),
      format!("Audio prezzo massimo raggiunto: {}", app_ctx.options.audio_max_price_file_name),
      format!("Audio percentuale ribasso toccata: {}", app_ctx.options.audio_down_price_file_name),
      format!("Audio percentuale rialzo toccata: {}", app_ctx.options.audio_up_price_file_name),
      String::from("INDIETRO"),
  ];

  menu_items
}

// TODO: INCAPSULARE IN APPCONTEXT O ALTROVE. SENZA AUDIO. IL PROBLEMA è SALVARE LE OPZIONI
fn toggle_sound(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  app_ctx.options.menu_sound_enabled = !app_ctx.options.menu_sound_enabled;
  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
      eprintln!("Errore nel salvataggio: {}\r", e);
  }
  if app_ctx.options.menu_sound_enabled == true {
    if let Some(menu_sample) = app_ctx.audio_assets.get("menu_click") {
      app_ctx.audio_manager.play_feedback(menu_sample.clone());
    }
  }
  Ok(())
}

// MENU PRINCIPALE
pub fn show_menu(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>>{
  let mut stdout = io::stdout();

  queue!(stdout, EnterAlternateScreen)?;
  terminal::enable_raw_mode()?;
  queue!(stdout, Hide)?;
  stdout.flush()?;

  let menu_items: Vec<String> = vec![
    "Avvia".to_string(),
    "Opzioni".to_string(),
    "ESCI".to_string()];
  let mut selected = 0;

  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 1, "Menu Principale")?;

  'main_loop: loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
          continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
            selected -= 1;
            app_ctx.play_menu_sound("menu_move");

        },
        KeyCode::Down if selected + 1 < menu_items.len() => {
            selected += 1;
            app_ctx.play_menu_sound("menu_move");
        },
        KeyCode::Enter => {
            app_ctx.play_menu_sound("menu_click");
            let choice = menu_items[selected].as_str();
            match choice {
                "Avvia" => {
                  // TODO: INCAPSULARE IN FUNZIONE
                  execute!(
                      stdout,
                      terminal::Clear(ClearType::All),
                      cursor::MoveTo(0, 0)
                  )?;

                  if app_ctx.options.logs_enabled {
                    let logger = Logger::new();
                    app_ctx.set_logger(logger);
                  } else {
                    app_ctx.clear_logger();
                  }

                  if app_ctx.options.audio_min_price_file_name.is_empty() {
                      app_ctx
                          .set_audio_asset("min_price", "")
                          .unwrap();
                  } else {
                      app_ctx
                          .set_audio_asset("min_price", format!("./audio/monitor/min/{}", app_ctx.options.audio_min_price_file_name))
                          .unwrap();
                  }

                  if app_ctx.options.audio_max_price_file_name.is_empty() {
                      app_ctx
                          .set_audio_asset("max_price", "")
                          .unwrap();
                  } else {
                      app_ctx
                          .set_audio_asset("max_price", format!("./audio/monitor/max/{}", app_ctx.options.audio_max_price_file_name))
                          .unwrap();
                  }

                  if app_ctx.options.audio_down_price_file_name.is_empty() {
                      app_ctx
                          .set_audio_asset("down_price_perc", "")
                          .unwrap();
                  } else {
                      app_ctx
                          .set_audio_asset("down_price_perc", format!("./audio/monitor/min/{}", app_ctx.options.audio_down_price_file_name))
                          .unwrap();
                  }

                  if app_ctx.options.audio_up_price_file_name.is_empty() {
                      app_ctx
                          .set_audio_asset("up_price_perc", "")
                          .unwrap();
                  } else {
                      app_ctx
                          .set_audio_asset("up_price_perc", format!("./audio/monitor/max/{}", app_ctx.options.audio_up_price_file_name))
                          .unwrap();
                  }

                  /*match generic_mod::launch(app_ctx) { // TODO:
                    Ok(()) => { let _ = generic_mod::launch(app_ctx).unwrap(); }
                    Err(e) => eprintln!("Errore durante l'avvio del browser: {}", e),
                  }*/

                  let token_info = TOKENS.get(&app_ctx.options.token_id).unwrap(); // TODO: MANCA DEFAULT

                  if CONFIG_RUN.launch_type == 1 {
                    app_ctx.options.url = format!("https://jup.ag/swap?sell={}&buy={}", token_info.sell_addr, token_info.buy_addr);
                    if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                    }
                    let _ = generic_mod::launch_1(app_ctx).unwrap();
                  } else {
                    app_ctx.options.url = format!("https://jup.ag/tokens/{}", token_info.mint_addr);
                    if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                    }
                    let _ = generic_mod::launch_2(app_ctx).unwrap();
                  }

                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 1, "Menu Principale")?;
                }
                "Opzioni" => {
                  run_options(app_ctx)?;
                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 1, "Menu Principale")?;
                }
                "ESCI" => break 'main_loop,
                _ => {}
            }
        },
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 1)?;
        },
        KeyCode::Esc => break 'main_loop,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &menu_items, old, selected, MENU_START_ROW)?;
      }

    }
  }

  terminal::disable_raw_mode()?;
  queue!(stdout, Show, terminal::LeaveAlternateScreen)?;
  stdout.flush()?;

  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
      eprintln!("Errore durante il salvataggio finale della configurazione: {}\r", e);
  }

  Ok(())
}

/// MENU OPZIONI
pub fn run_options(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  queue!(stdout, Hide)?;
  stdout.flush()?;

  let menu_items: Vec<String> = vec![
      String::from("Audio"),
      String::from("Monitoraggio"),
      String::from("INDIETRO"),
  ];
  let mut selected = 0;

  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni")?;

  // Svuota eventuali eventi residui
  while event::poll(Duration::from_millis(10))? {
      let _ = event::read()?;
  }

  loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
          continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
          selected -= 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Down if selected + 1 < menu_items.len() => {
          selected += 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Enter => {
          if selected == menu_items.len() - 1 {
            app_ctx.play_menu_sound("menu_click");
            break;
          } else {
            match selected {
              0 => {
                app_ctx.play_menu_sound("menu_click");
                run_options_audio(app_ctx)?;
                draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni")?;
              }
              1 => {
                app_ctx.play_menu_sound("menu_click");
                run_options_monitoring(app_ctx)?;
                draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni")?;
              }
              _ => {}
            }
          }
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 2)?;
        }
        KeyCode::Esc => break,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &menu_items, old, selected, MENU_START_ROW)?;
      }

    }
  }

  Ok(())

}

/// MENU OPZIONI -> AUDIO
pub fn run_options_audio(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  queue!(stdout, Hide)?;
  stdout.flush()?;

  let lang = app_ctx.options.locale.clone();

  let mut menu_items = set_menu_items_options(app_ctx, &lang);

  let mut selected = 0;

  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;

  // Svuota eventuali eventi residui
  while event::poll(Duration::from_millis(10))? {
      let _ = event::read()?;
  }

  loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
          continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
          selected -= 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Down if selected + 1 < menu_items.len() => {
          selected += 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 'd' => {
          app_ctx.play_menu_sound("menu_click");
          app_ctx.options.reset_fields(|opt, d| {
              opt.sound_enabled = d.sound_enabled;
              opt.audio_min_price_file_name = d.audio_min_price_file_name;
              opt.audio_max_price_file_name = d.audio_max_price_file_name;
              opt.audio_down_price_file_name = d.audio_down_price_file_name;
              opt.audio_up_price_file_name = d.audio_up_price_file_name;
          });
          menu_items = set_menu_items_options(app_ctx, &lang);
          draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;
          if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
              eprintln!("Errore nel salvataggio: {}\r", e);
          }
        }
        KeyCode::Enter => {
          if selected == menu_items.len() - 1 {
            app_ctx.play_menu_sound("menu_click");
            break;
          } else {
            match selected {
              0 => {
                  app_ctx.options.sound_enabled = !app_ctx.options.sound_enabled;
                  menu_items[0] = format!("Abilita suono: {}", if app_ctx.options.sound_enabled { "Si" } else { "No" });
                  app_ctx.play_menu_sound("menu_click");
                  redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
                  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                  }
              }
              1 => {
                  app_ctx.play_menu_sound("menu_click");
                  run_select_audio(app_ctx, "min")?;
                  menu_items[1] = format!("Audio prezzo minimo raggiunto: {}", app_ctx.options.audio_min_price_file_name);
                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;
              }
              2 => {
                  app_ctx.play_menu_sound("menu_click");
                  run_select_audio(app_ctx, "max")?;
                  menu_items[2] = format!("Audio prezzo massimo raggiunto: {}", app_ctx.options.audio_max_price_file_name);
                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;
              }
              3 => {
                  app_ctx.play_menu_sound("menu_click");
                  run_select_audio(app_ctx, "down_perc")?;
                  menu_items[3] = format!("Audio percentuale ribasso toccata: {}", app_ctx.options.audio_down_price_file_name);
                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;
              }
              4 => {
                  app_ctx.play_menu_sound("menu_click");
                  run_select_audio(app_ctx, "up_perc")?;
                  menu_items[4] = format!("Audio percentuale rialzo raggiunta: {}", app_ctx.options.audio_up_price_file_name);
                  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Audio")?;
              }
              _ => {}
            }
          }
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 2)?;
        }
        KeyCode::Esc => break,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &menu_items, old, selected, MENU_START_ROW)?;
      }

    }
  }

  Ok(())

}

/// MENU OPZIONI -> MONITORAGGIO
pub fn run_options_monitoring(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  queue!(stdout, Hide)?;
  stdout.flush()?;

  let lang = app_ctx.options.locale.clone();

  let (mut menu_items, _) = set_menu_items_monitoring(app_ctx, &lang);

  let mut selected = 0;

  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Monitoraggio")?;

  // Svuota eventuali eventi residui
  while event::poll(Duration::from_millis(10))? {
      let _ = event::read()?;
  }

  loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
          continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
          selected -= 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Down if selected + 1 < menu_items.len() => {
          selected += 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 'd' => {
          app_ctx.play_menu_sound("menu_click");
          app_ctx.options.reset_fields(|opt, d| {
              opt.chrome_path = d.chrome_path;
              opt.view_type = d.view_type;
              opt.monitoring_interval_secs = d.monitoring_interval_secs;
              opt.logs_enabled = d.logs_enabled;
              opt.stop_when_price_reached = d.stop_when_price_reached;
              opt.token_id = d.token_id;
              opt.token_quantity = d.token_quantity;
              opt.min_price = d.min_price;
              opt.max_price = d.max_price;
              opt.view_type = d.view_type;
              opt.down_price_perc = d.down_price_perc;
              opt.up_price_perc = d.up_price_perc;
          });
          menu_items = set_menu_items_monitoring(app_ctx, &lang).0;
          draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Monitoraggio")?;
          if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
              eprintln!("Errore nel salvataggio: {}\r", e);
          }
        }
        KeyCode::Enter => {
          if selected == menu_items.len() - 1 {
            app_ctx.play_menu_sound("menu_click");
            break;
          } else {
            match selected {
              0 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci percorso: ")?;
                // TODO: CHECK EMPTY OR ESC
                //if let Ok(v) = resp.parse() {
                let v = resp.parse::<String>().unwrap();
                  menu_items[0] = format!("Percorso Chrome o Chromium: {}", v);
                  app_ctx.options.chrome_path = v;
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                //}

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              1 => {
                  app_ctx.options.view_type = app_ctx.options.view_type.next();
                  menu_items[1] = format!("Visualizzazione: {}", app_ctx.options.view_type.label(&lang));
                  app_ctx.play_menu_sound("menu_click");
                  redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
                  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                  }
              }
              2 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci intervallo di monitoraggio (s): ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.monitoring_interval_secs = v;
                  menu_items[2] = format!("Intervallo di monitoraggio (s): {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              3 => {
                  app_ctx.options.logs_enabled = !app_ctx.options.logs_enabled;
                  menu_items[3] = format!("Abilita logs: {}", if app_ctx.options.logs_enabled { "Si" } else { "No" });
                  app_ctx.play_menu_sound("menu_click");
                  redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
                  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                  }
              }
              4 => {
                  app_ctx.options.stop_when_price_reached = !app_ctx.options.stop_when_price_reached;
                  menu_items[4] = format!("Interrompi a prezzo minimo o massimo: {}", if app_ctx.options.stop_when_price_reached { "Si" } else { "No" });
                  app_ctx.play_menu_sound("menu_click");
                  redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
                  if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                      eprintln!("Errore nel salvataggio: {}\r", e);
                  }
              }
              5 => {
                app_ctx.play_menu_sound("menu_click");
                run_select_token(app_ctx)?;
                let token_info = TOKENS.get(&app_ctx.options.token_id).unwrap(); // TODO: MANCA DEFAULT
                let token_name = &token_info.symbol;
                menu_items[5] = format!("Token: {}", token_name);
                draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Monitoraggio")?;
              }
              6 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci quantità: ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.token_quantity = v;
                  menu_items[6] = format!("Quantità: {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              7 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci prezzo minimo: ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.min_price = v;
                  menu_items[7] = format!("Prezzo minimo: {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              8 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci prezzo massimo: ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.max_price = v;
                  menu_items[8] = format!("Prezzo massimo: {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              9 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci ribasso %: ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.down_price_perc = v;
                  menu_items[9] = format!("Ribasso %: {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              10 => {
                let resp = prompt_and_read(&mut stdout, "Inserisci rialzo %: ")?;
                if let Ok(v) = resp.parse() {
                  app_ctx.options.up_price_perc = v;
                  menu_items[10] = format!("Rialzo %: {}", v);
                  app_ctx.play_menu_sound("menu_click");
                  Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options).ok();
                }

                redraw_selection(&mut stdout, &menu_items, selected, selected, MENU_START_ROW)?;
              }
              _ => {}
            }
          }
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 2)?;
        }
        KeyCode::Esc => break,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &menu_items, old, selected, MENU_START_ROW)?;
      }

    }
  }

  Ok(())

}

/// MENU OPZIONI -> MONITORAGGIO -> TOKEN
pub fn run_select_token(app_ctx: &mut AppContext) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  queue!(stdout, Hide)?;
  stdout.flush()?;

  // TODO:
  /*let mut menu_items: Vec<String> = TOKENS
    .values()                                         // &TokenInfo
    .map(|info| format!("{} - {}", info.symbol, info.full_name))
    .collect();*/

  let mut symbols: Vec<String> = TOKENS
    .keys()
    .cloned()
    .collect();

  symbols.sort_by_key(|sym| {
      TOKENS.get(sym).unwrap().symbol.clone()
  });

  let mut menu_items: Vec<String> = symbols.clone();

  menu_items.push(String::from("INDIETRO"));

  let mut selected = symbols
    .iter()
    .position(|s| s == &app_ctx.options.token_id)
    .unwrap_or(0);

  draw_full_menu(&mut stdout, app_ctx, &menu_items, selected, print_header, 2, "Opzioni >> Monitoraggio >> Seleziona Token")?;

  // Svuota eventuali eventi residui
  while event::poll(Duration::from_millis(10))? {
      let _ = event::read()?;
  }

  loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
        continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
          selected -= 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Down if selected + 1 < menu_items.len() => {
          selected += 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Enter => {
          if selected == menu_items.len() - 1 {
            app_ctx.play_menu_sound("menu_click");
            break;
          } else {
            app_ctx.options.token_id = symbols[selected].clone();
            app_ctx.play_menu_sound("menu_click");
            if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
                eprintln!("Errore nel salvataggio: {}\r", e);
            }
            break;
          }
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 2)?;
        }
        KeyCode::Esc => break,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &menu_items, old, selected, MENU_START_ROW)?;
      }

    }
  }

  Ok(())

}

pub fn run_select_audio(app_ctx: &mut AppContext, min_or_max: &str) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  queue!(stdout, Hide)?;
  stdout.flush()?;

  let path = if min_or_max == "min" || min_or_max == "down_perc" {
    format!("audio/monitor/{}", "min")
  } else {
    format!("audio/monitor/{}", "max")
  };

  let mut audio_files = AudioAssets::get_audio_files(&path).unwrap_or_else(|_| vec![]);
  audio_files.insert(0, "(nessuno)".to_string());

  if audio_files.len() == 1 {
      println!("Nessun file audio trovato nella cartella 'audio'.");
      return Ok(());
  }

  let mut selected: usize = {
    let target = if min_or_max == "max" {
      &app_ctx.options.audio_max_price_file_name
    } else if min_or_max == "min" {
      &app_ctx.options.audio_min_price_file_name
    } else if min_or_max == "down_perc" {
      &app_ctx.options.audio_down_price_file_name
    } else {
      &app_ctx.options.audio_up_price_file_name
    };

    if target.is_empty() {
      0
    } else {
      audio_files
        .iter()
        .position(|name| name == target)
        .unwrap_or(0)
    }
  };

  audio_files.push(String::from("INDIETRO"));

  draw_full_menu(&mut stdout, app_ctx, &audio_files, selected, print_header, 3, "Opzioni >> Audio >> Seleziona File Audio")?;

  // Svuota eventuali eventi residui.
  while event::poll(Duration::from_millis(10))? {
      let _ = event::read()?;
  }

  loop {
    if let Event::Key(key_event) = event::read()? {
      let old = selected;
      if key_event.kind != KeyEventKind::Press {
        continue;
      }
      match key_event.code {
        KeyCode::Up if selected > 0 => {
          selected -= 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Down if selected + 1 < audio_files.len() => {
          selected += 1;
          app_ctx.play_menu_sound("menu_move");
        }
        KeyCode::Enter => {
          if selected == audio_files.len() - 1 {
            app_ctx.play_menu_sound("menu_click");
            break;
          } else {
            app_ctx.play_menu_sound("menu_click");
            let selected_file = audio_files[selected].clone();
            let selected_file_name = if selected == 0 { "".to_string() } else { Some(selected_file).unwrap() };
            if min_or_max == "max" {
              app_ctx.options.audio_max_price_file_name = selected_file_name;
            } else if min_or_max == "min" {
              app_ctx.options.audio_min_price_file_name = selected_file_name;
            } else if min_or_max == "down_perc" {
              app_ctx.options.audio_down_price_file_name = selected_file_name;
            } else {
              app_ctx.options.audio_up_price_file_name = selected_file_name;
            }
            if let Err(e) = Options::save(&CONFIG_RUN.config_file_name, &app_ctx.options) {
              eprintln!("Errore nel salvataggio: {}\r", e);
            }
            break;
          }
        }
        KeyCode::Char(c) if c.to_ascii_lowercase() == 's' => {
          toggle_sound(app_ctx).unwrap();
          redraw(&mut stdout, app_ctx, 9, print_commands, 3)?;
        },
        KeyCode::Char(c) if c.to_ascii_lowercase() == 'r' => {
          let selected_file = &audio_files[selected];
          let full_path = format!("{}/{}", path, selected_file);
          app_ctx.audio_manager.play_audio(&full_path);
        }
        KeyCode::Esc => break,
        _ => {}
      }

      if selected != old {
        redraw_selection(&mut stdout, &audio_files, old, selected, MENU_START_ROW)?;
      }

    }
  }

  Ok(())

}