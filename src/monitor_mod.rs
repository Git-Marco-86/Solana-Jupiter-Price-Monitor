/**
 * v4.0.1 27/09/2025
 * Author: Marco Maffei
 *
 */

// <custom
use crate::AppContext;
use crate::tokens::TOKENS;
use crate::view_type::ViewType;
use crate::monitor_state::MonitorState;
use crate::notification_mod;
use crate::gui::{show_dialog, show_simple_dialog, shade_percent, shade_fixed};
use crate::spinner::Spinner;
// </custom

use std::io::{self, Write, Stdout};
use std::thread;
use std::time::{Duration, Instant};


use crossbeam_channel::{select, unbounded, Sender, Receiver};

use headless_chrome::Tab;

use crossterm::{
    event::{self, read, poll, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{Clear, ClearType},
    cursor,
    queue,
    style::Print,
    QueueableCommand,
};

use chrono::Local;

use colored::*;


enum MonitorAction {
  Continue,
  Quit,
}

pub fn start_monitor_1(app_ctx: &mut AppContext, tab: &Tab, spinner: Spinner) -> Result<(), Box<dyn std::error::Error>> {
  let (tx_keys, rx_keys) = unbounded::<KeyCode>();
  let (tx_stop, rx_stop) = unbounded::<()>();

  let handle = spawn_input_worker(tx_keys.clone(), rx_stop);

  let mut state = MonitorState::new();
  let interval = Duration::from_secs(app_ctx.options.monitoring_interval_secs);
  let mut last_scrape = Instant::now();
  let mut stdout = io::stdout();

  // TODO: L'ERRORE NON è GESTITO CORRETTAMENTE PERCHé NON VISUALIZZA IL MESSAGGIO SE NON TROVA L'ELEMENTO
  //let update_element = tab.wait_for_element(&app_ctx.options.update_element)?;
  let update_element = match tab.wait_for_element(&app_ctx.options.update_element) {
      Ok(el) => el,
      Err(e) => {
          let msg = format!("Timeout: l'elemento di AGGIORNAMENTO non è stato trovato: {}\rErrore: {}\r",
                            &app_ctx.options.update_element, e);
          queue!(stdout, cursor::Hide, Print(&msg))?;
          stdout.flush()?;
          return Err(e.into());
      }
  };

  spinner.stop();

  queue!(
        stdout,
        cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(ClearType::All)
    )?;

  //let output_price_el = tab.wait_for_element(&app_ctx.options.output_element)?; // TODO: è UN ELEMENTO STATICO CHE NON AGGIORNA VALORI DOPO IL CLICK DI UPDATE

  loop {

    update_element.click()?; // at the top for less latency

    select! {
      recv(rx_keys) -> msg => {
        if let Ok(code) = msg {
          if let MonitorAction::Quit = handle_monitor_key(
            code, &mut stdout, app_ctx, &mut state, &tx_stop
          )? {
            handle.join().unwrap();
            break;
          }
        }
      },
      default(interval.saturating_sub(last_scrape.elapsed())) => {
        if !state.is_paused {
          let output_price_el = match tab.wait_for_element_with_custom_timeout(
            &app_ctx.options.output_element,
            Duration::from_secs(app_ctx.options.response_timeout_secs),
          ) {
            Ok(element) => element,
            Err(err) => {
                //eprintln!("Timeout: l'elemento non è stato trovato dopo {} secondi: {}\r", &app_ctx.options.response_timeout_secs, err); // TODO:
                let message = format!("Timeout: l'elemento di OUTPUT non è stato trovato dopo {} secondi: {}\r", &app_ctx.options.response_timeout_secs, err);
                queue!(
                    stdout,
                    Print(message),
                )?;
                stdout.flush()?;
                thread::sleep(Duration::from_secs(app_ctx.options.response_timeout_secs));
                continue;
            }
          };

          match output_price_el.get_attributes() {
            Ok(Some(attrs_vec)) => {
              if let Some(idx) = attrs_vec.iter().position(|attr| attr == "value") {
                if idx + 1 < attrs_vec.len() {
                  let value = attrs_vec[idx + 1]
                    .replace(',', ".")
                    .replace('\u{00A0}', "")
                    .chars()
                    .filter(|c| c.is_digit(10) || *c == '.' || *c == '-')
                    .collect::<String>();

                  if print_output(&mut stdout, app_ctx, &mut state, "".to_string(), value)?{
                    break;
                  }

                } else {
                  // eprintln!("Chiave 'value' trovata ma nessun valore associato.\r"); // TODO:
                  queue!(
                      stdout,
                      Print("Chiave 'value' trovata ma nessun valore associato.\r"),
                  )?;
                  stdout.flush()?;
                }
              } else {
                //eprintln!("L'attributo 'value' non è presente.\r"); // TODO:
                queue!(
                    stdout,
                    Print("L'attributo 'value' non è presente.\r"),
                )?;
                stdout.flush()?;
              }
            }
            Ok(None) => {
                //eprintln!("Nessun attributo trovato.\r"); // TODO:
                queue!(
                    stdout,
                    Print("Nessun attributo trovato.\r"),
                )?;
                stdout.flush()?;
            }
            Err(e) => {
                //eprintln!("Errore nel recuperare gli attributi: {}\r", e); // TODO:
                let message = format!("Errore nel recuperare gli attributi: {}\r", e);
                queue!(
                    stdout,
                    Print(message),
                )?;
                stdout.flush()?;
            }
          }

          last_scrape = Instant::now();
        }
      }
    }
  }

  Ok(())

}

pub fn start_monitor_2(app_ctx: &mut AppContext, tab: &Tab, spinner: Spinner) -> Result<(), Box<dyn std::error::Error>> {
  let (tx_keys, rx_keys) = unbounded::<KeyCode>();
  let (tx_stop, rx_stop) = unbounded::<()>();

  let handle = spawn_input_worker(tx_keys.clone(), rx_stop);

  let mut state = MonitorState::new();
  let interval = Duration::from_secs(app_ctx.options.monitoring_interval_secs);
  let mut last_scrape = Instant::now();
  let mut stdout = io::stdout();

  spinner.stop();

  queue!(
        stdout,
        cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(ClearType::All)
    )?;

  loop {

    select! {
      recv(rx_keys) -> msg => {
        if let Ok(code) = msg {
          if let MonitorAction::Quit = handle_monitor_key(
            code, &mut stdout, app_ctx, &mut state, &tx_stop
          )? {
            handle.join().unwrap();
            break;
          }
        }
      },
      default(interval.saturating_sub(last_scrape.elapsed())) => {
        if !state.is_paused {
          let expr = format!(
            r#"
              (() => {{
                const el = document.querySelector("{sel}");
                if (!el) return "";
                const txt = el.innerText.trim();
                return txt;
              }})()
            "#,
            sel = &app_ctx.options.output_element
          );

          let raw_text = loop {
            let val = tab.evaluate(&expr, false)?
                  .value
                  .and_then(|v| v.as_str().map(|s| s.to_string()))
                  .unwrap_or_default();
            if !val.is_empty() {
              break val;
            }
            if Instant::now() > Instant::now() + Duration::from_secs(app_ctx.options.response_timeout_secs) {
              queue!(stdout, Print(format!(
                "Timeout: `{}` non popolato\n", &app_ctx.options.output_element
              )))?;
              stdout.flush()?;
              continue;
            }
            thread::sleep(Duration::from_millis(100));
          };

          let value: String = raw_text
            .replace('\u{00A0}', "")
            .replace(',', ".")
            .replace('$', "")
            .chars()
            .filter(|c| c.is_digit(10) || *c=='.' || *c=='-')
            .collect();

          if print_output(&mut stdout, app_ctx, &mut state, "".to_string(), value)?{
            break;
          }

          last_scrape = Instant::now();
        }
      }
    }
  }

  Ok(())

}

fn spawn_input_worker(
    tx_keys: Sender<KeyCode>,
    stop_rx: Receiver<()>
) -> thread::JoinHandle<()> {
  thread::spawn(move || {
    loop {
      select! {
        recv(stop_rx) -> _ => break,
        default(Duration::from_millis(20)) => {
          if poll(Duration::ZERO).unwrap() {
            if let Event::Key(KeyEvent { code, kind, .. }) = read().unwrap() {
              if kind == KeyEventKind::Press {
                match code {
                    KeyCode::Esc
                  | KeyCode::F(1)
                  | KeyCode::F(2)
                      => {
                          let _ = tx_keys.send(code);
                      }
                  | KeyCode::Char(c) if ['p', 'c', 'r', 's', 'v']
                    .contains(&c.to_ascii_lowercase())
                      => {
                        let _ = tx_keys.send(KeyCode::Char(c.to_ascii_lowercase()));
                      }
                  _ => {}
                }
              }
            }
          }
        }
      }
    }
  })
}

fn handle_monitor_key(
    code: KeyCode,
    stdout: &mut Stdout,
    app_ctx: &mut AppContext,
    state: &mut MonitorState,
    tx_stop: &Sender<()>,
) -> Result<MonitorAction, Box<dyn std::error::Error>> {
    let action = match code {
      KeyCode::Esc => {
        if ask_quit_confirm(stdout)? {
          let _ = tx_stop.send(());
          MonitorAction::Quit
        } else {
          MonitorAction::Continue
        }
      }
      KeyCode::F(1) => {
        show_commands(stdout)?;
        state.is_first_output = true;
        MonitorAction::Continue
      }
      KeyCode::F(2) => {
        show_parameters(stdout, app_ctx)?;
        state.is_first_output = true;
        MonitorAction::Continue
      }
      KeyCode::Char('p') => {
        pause(stdout, state)?;
        MonitorAction::Continue
      }
      KeyCode::Char('c') => {
        queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        stdout.flush()?;
        MonitorAction::Continue
      }
      KeyCode::Char('r') => {
        *state = MonitorState::new();
        MonitorAction::Continue
      }
      KeyCode::Char('s') => {
        toggle_sound(stdout, app_ctx)?;
        MonitorAction::Continue
      }
      KeyCode::Char('v') => {
        next_view(stdout, app_ctx)?;
        state.is_first_output = true;
        MonitorAction::Continue
      }
      _ => MonitorAction::Continue,
    };
    stdout.flush()?;
    Ok(action)
}

fn ask_quit_confirm(stdout: &mut Stdout) -> Result<bool, Box<dyn std::error::Error>> {
  queue!(
      stdout,
      cursor::Hide,
      Print("\r\nVuoi interrompere il monitoraggio? (s/n)\r")
  )?;
  stdout.flush()?;

  while poll(Duration::ZERO)? { let _ = read()?; }

  loop {
    if let Event::Key(k) = read()? {
      if let KeyCode::Char(c) = k.code {
        match c.to_ascii_lowercase() {
          's' => {
            return Ok(true);
          }
          'n' => {
            queue!(
                stdout,
                Clear(ClearType::CurrentLine),
                cursor::MoveUp(1),
            )?;
            stdout.flush()?;
            return Ok(false);
          }
          _ => {}
        }
      }
    }
  }
}

fn show_commands(stdout: &mut Stdout) -> Result<bool, Box<dyn std::error::Error>> {
  let items = [
    " F2: parametri impostati",
    "  S: suono On/Off",
    "  P: pausa/riprendi",
    "  C: pulisci schermata",
    "  R: resetta valori",
    "  V: cambia visualizzazione",
    "ESC: interrompi",
    "",
    " F1: indietro/comandi",
  ];
  let _ = show_simple_dialog(stdout, "COMANDI", &items, KeyCode::F(1))?;

  Ok(true)
}

fn show_parameters(stdout: &mut Stdout, app_ctx: &mut AppContext) -> Result<bool, Box<dyn std::error::Error>> {

  let token_info = TOKENS.get(&app_ctx.options.token_id).unwrap(); // TODO: MANCA DEFAULT
  let token_name = &token_info.symbol;

  let monitoring_interval_secs = format!("Intervallo di monitoraggio (s): {}", app_ctx.options.monitoring_interval_secs);
  let token = format!("Token: {}", token_name);
  let quantity = format!("Quantità: {}", app_ctx.options.token_quantity);
  let min_price = format!("Prezzo minimo: {}", app_ctx.options.min_price);
  let max_price = format!("Prezzo massimo: {}", app_ctx.options.max_price);
  let up_price_perc = format!("Ribasso %: {}", app_ctx.options.up_price_perc);
  let down_price_perc = format!("Rialzo %: {}", app_ctx.options.down_price_perc);
  let items: [&str; 9] = [
      monitoring_interval_secs.as_str(),
      token.as_str(),
      quantity.as_str(),
      min_price.as_str(),
      max_price.as_str(),
      up_price_perc.as_str(),
      down_price_perc.as_str(),
      "",
      "F2: indietro",
    ];

  let _ = show_simple_dialog(stdout, "PARAMETRI", &items, KeyCode::F(2))?;

  Ok(true)
}

fn toggle_sound(stdout: &mut Stdout, app_ctx: &mut AppContext) -> Result<bool, Box<dyn std::error::Error>> {
  app_ctx.options.sound_enabled = !app_ctx.options.sound_enabled;
  if app_ctx.options.view_type == ViewType::FIXED {
    let text = format!("(F1: visualizza comandi, Suono: {})\r\n", if app_ctx.options.sound_enabled { "On" } else { "Off" });
    queue!(
      stdout,
      cursor::MoveTo(0, 8),
      Clear(ClearType::CurrentLine),
      Print(text),
    )?;
  }

  Ok(true)
}

fn next_view(stdout: &mut Stdout, app_ctx: &mut AppContext) -> Result<bool, Box<dyn std::error::Error>> {
  app_ctx.options.view_type = app_ctx.options.view_type.next();
  queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
  stdout.flush()?;

  Ok(true)
}

fn pause(stdout: &mut Stdout, state: &mut MonitorState) -> Result<bool, Box<dyn std::error::Error>> {
  state.is_paused = !state.is_paused;

  if state.is_paused {
    queue!(
        stdout,
        cursor::Hide,
        Print("\r\n-- PAUSE --\r")
    )?;
  } else {
    queue!(
        stdout,
        Clear(ClearType::CurrentLine),
        cursor::MoveUp(1),
    )?;
  }

  stdout.flush()?;
  Ok(true)
}

fn print_output(stdout: &mut Stdout, app_ctx: &mut AppContext, state: &mut MonitorState, raw_text: String, value: String) -> Result<bool, Box<dyn std::error::Error>> {

  let value_f: f32 = match value.parse::<f32>() {
    Ok(v) => v,
    Err(_e) => {
        let detail = format!(
            "cannot parse float from empty string; raw_text={:?}, value={:?}",
            raw_text, value
          );
          return Err(detail.into());
    }
  };

  if state.min_price_reached == 0.0 {
    state.min_price_reached = value_f;
  }

  if value_f < state.min_price_reached {
    state.min_price_reached = value_f;
  }

  if value_f > state.max_price_reached {
    state.max_price_reached = value_f;
  }

  let mut is_reached_down = false;
  let mut is_reached_up = false;
  let mut current_perc_diff = 0.0;
  let mut is_price_changed = false;

  let (colored_value, colored_current_perc_diff, colored_total_perc_diff) = if let Some(prev) = state.prev_value_f {
    current_perc_diff = 100.0-(prev * 100.0 / value_f);
    state.total_perc_diff += current_perc_diff;

    if current_perc_diff != 0.0 {
      if app_ctx.options.up_price_perc != 0.0 && current_perc_diff >= app_ctx.options.up_price_perc {
        app_ctx.play_sound("up_price_perc");
        is_reached_up = true;
      } else if app_ctx.options.down_price_perc != 0.0 && current_perc_diff <= -app_ctx.options.down_price_perc {
        app_ctx.play_sound("down_price_perc");
        is_reached_down = true;
      }
    }

    let colored_total_perc_diff = if state.total_perc_diff < 0.0 || state.total_perc_diff > 0.0 {
      if cfg!(windows) {
        shade_fixed(state.total_perc_diff)
      } else {
        shade_percent(state.total_perc_diff, app_ctx.options.up_price_perc.max(app_ctx.options.down_price_perc))
      }
    } else {
      shade_fixed(0.0)
    };

    if value_f < prev {
      is_price_changed = true;
      (format!("{}", value).red(), format!("{}%", current_perc_diff).red(), colored_total_perc_diff)
    } else if value_f > prev {
      is_price_changed = true;
      (format!("{}", value).green(), format!("+{}%", current_perc_diff).green(), colored_total_perc_diff)
    } else {
      (format!("{}", value).yellow(), format!("{}", "").yellow(), colored_total_perc_diff)
    }
  } else {
      (format!("{}", value).yellow(), format!("{}", "").yellow(), shade_fixed(0.0))
  };

  state.prev_value_f = Some(value_f);

  let token_info = TOKENS.get(&app_ctx.options.token_id).unwrap(); // TODO: MANCA DEFAULT
  let token_name = &token_info.symbol;

  let colored_token_name = format!("{}", token_name).cyan();

  let now = Local::now();
  let timestamp = now.format(&app_ctx.options.datetime_format).to_string();
  let base_output = if app_ctx.options.view_type == ViewType::SEQUENTIAL {
        format!("{} >>> Il prezzo attuale di {} è: {:^16} ({:^16}) ({:^16}) (Min: {:^16}) (Max: {:^16}) ----- (F1: visualizza comandi, Suono: {})\r",
                timestamp,
                colored_token_name,
                colored_value,
                colored_current_perc_diff,
                colored_total_perc_diff,
                state.min_price_reached,
                state.max_price_reached,
                if app_ctx.options.sound_enabled { "On" } else { "Off" },
            )
      } else {
        format!("Data/ora: {}\r\n\
                Token: {}\r\nPrezzo: {}\r\n\
                Differenza precedente (%): {}\r\n\
                Differenza totale (%): {}\r\n\
                Prezzo minimo: {}\r\n\
                Prezzo massimo: {}\r\n\r\n\
                (F1: visualizza comandi, Suono: {})\r\n",
                timestamp,
                colored_token_name,
                colored_value,
                colored_current_perc_diff,
                colored_total_perc_diff,
                state.min_price_reached,
                state.max_price_reached,
                if app_ctx.options.sound_enabled { "On" } else { "Off" },
            )
    };

  let mut styled: ColoredString = base_output.as_str().into();

  if is_reached_down {
      styled = styled.on_yellow();
  }

  if is_reached_up {
      styled = styled.on_blue();
  }

  let log_text = format!("{};{};{};{};{};{};{};",
        timestamp,
        token_name,
        value,
        current_perc_diff,
        state.total_perc_diff,
        state.min_price_reached,
        state.max_price_reached,
    );

  if app_ctx.options.logs_enabled {
    if let Some(logger) = &app_ctx.logger {
      logger.log(log_text);
    }
  }

  match app_ctx.options.view_type {
    ViewType::SEQUENTIAL => {
      stdout.queue(Print(styled))?;
      stdout.queue(Print("\r\n"))?;
    }
    ViewType::FIXED => {
      if state.is_first_output || is_price_changed {
          stdout.queue(Clear(ClearType::All))?;
          stdout.queue(cursor::MoveTo(0, 0))?;
          stdout.queue(Print(styled))?;
      } else {
        queue!(
          stdout,
          cursor::MoveTo(10, 0), // only datetime
          Print(timestamp),
          cursor::MoveTo(0, 9)
        )?;
      }
    }
  }

  state.is_first_output = false;

  stdout.flush()?;

  if app_ctx.options.max_price != 0.0 && value_f > app_ctx.options.max_price {

    app_ctx.play_sound("max_price");

    let text = format!("\r\nIl valore ha superato la soglia massima: {}\r\n", value).on_cyan();

    queue!(
        stdout,
        Print(&text),
        cursor::Hide,
    )?;
    stdout.flush()?;

    if let Err(e) = notification_mod::send_notification("Avviso Monitoraggio", &text) {
      //eprintln!("Errore nell'inviare la notifica: {}\r", e); // TODO:
      let message = format!("Errore nell'inviare la notifica: {}\r", e);
      queue!(
          stdout,
          Print(message),
          cursor::Hide,
      )?;
      stdout.flush()?;
    }

    queue!(
      stdout,
      cursor::Hide,
    )?;
    stdout.flush()?;

    if app_ctx.options.stop_when_price_reached {
      queue!(
          stdout,
          cursor::Hide,
          Print("\r\n\r\nPremi ESC per tornare al menu principale...\r"),
      )?;
      stdout.flush()?;

      loop {
        if event::poll(Duration::from_millis(10))? {
          if let Event::Key(key_event) = event::read()? {
            if key_event.code == KeyCode::Esc {
              return Ok(true);
            }
          }
        }
      }
    }
  } else if app_ctx.options.min_price != 0.0 && value_f < app_ctx.options.min_price {

    app_ctx.play_sound("min_price");

    let text = format!("\r\nIl valore ha superato la soglia minima: {}\r\n", value).on_magenta();

    queue!(
        stdout,
        Print(&text),
        cursor::Hide,
    )?;
    stdout.flush()?;

    if let Err(e) = notification_mod::send_notification("Monitoraggio", &text) {
      //eprintln!("Errore nell'inviare la notifica: {}\r", e); // TODO:
      let message = format!("Errore nell'inviare la notifica: {}\r", e);
      queue!(
          stdout,
          Print(message),
          cursor::Hide,
      )?;
      stdout.flush()?;
    }

    queue!(
      stdout,
      cursor::Hide,
    )?;
    stdout.flush()?;

    if app_ctx.options.stop_when_price_reached {
      queue!(
          stdout,
          cursor::Hide,
          Print("\r\n\r\nPremi ESC per tornare al menu principale...\r"),
      )?;
      stdout.flush()?;

      loop {
        if event::poll(Duration::from_millis(10))? {
          if let Event::Key(key_event) = event::read()? {
            if key_event.code == KeyCode::Esc {
              return Ok(true);
            }
          }
        }
      }
    }
  }

  return Ok(false);

}