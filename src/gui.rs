/**
 * v1.11.1 29/01/2026
 * Author: Marco Maffei
 *
 */

use crate::Options;
use crate::AppContext;

use std::io;
use std::io::Stdout;
use std::error::Error;
use std::io::Write;

use colored::Colorize;

use crossterm::{
    terminal::{self, Clear, ClearType},
    queue,
    style::{Print, Color, SetBackgroundColor, SetForegroundColor, ResetColor},
    cursor::{self, Hide, MoveTo},
    event::{Event, read, KeyCode, KeyEventKind},
};


const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

fn print_app_name(stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
    let header = format!("\
            ###############################################\r\n\
            #                                             #\r\n\
            #     SOLANA JUPITER PRICE MONITOR v{}     #\r\n\
            #    author: {}  #\r\n\
            #   website: {}                 #\r\n\
            #                                             #\r\n\
            ###############################################\r\n
        ",
        VERSION,
        AUTHORS,
        HOMEPAGE
    );

    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(ClearType::All),
        SetForegroundColor(Color::Blue),
        Print(header),
        Print("\r\n"),
    )?;
    Ok(())
}

pub fn print_header(stdout: &mut Stdout, options: &Options, n: u8) -> Result<(), Box<dyn Error>> {
    let _ = print_app_name(stdout);

    let text = if n == 1 {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (esci), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    } else if n == 2 {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (indietro), D (ripristina default), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    } else {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (indietro), R (riproduci), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    };

    queue!(
        stdout,
        Print(text),
    )?;

    Ok(())
}

pub fn print_commands(stdout: &mut Stdout, options: &Options, n: u8) -> Result<(), Box<dyn Error>> {
    let text = if n == 1 {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (esci), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    } else if n == 2 {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (indietro), D (ripristina default), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    } else {
        format!(
            "←↑↓→ (muovi), ↵ (seleziona), ESC (indietro), R (riproduci), S (suono {})\r\n\r\n",
            if options.menu_sound_enabled { "On" } else { "Off" }
        )
    };

    queue!(
        stdout,
        SetForegroundColor(Color::Blue),
        Print(text),
        ResetColor,
    )?;

    Ok(())
}

pub fn draw_full_menu<H>(
    stdout: &mut Stdout,
    app_ctx: &mut AppContext,
    items: &[String],
    selected: usize,
    mut header_fn: H,
    n: u8,
    title: &str,
) -> Result<(), Box<dyn Error>>
where
    H: FnMut(&mut Stdout, &Options, u8) -> Result<(), Box<dyn Error>>,
{

    terminal::enable_raw_mode()?;
    queue!(stdout, Hide)?;

    header_fn(stdout, &app_ctx.options, n)?;

    queue!(
        stdout,
        SetForegroundColor(Color::Yellow),
        Print(format!("--- {} ---\r\n\r\n", title)),
        ResetColor,
    )?;

    for (i, it) in items.iter().enumerate() {
        let marker = if i == selected { "\u{1F449} " } else { "   " };
        queue!(stdout, Print(marker), Print(it), Print("\r\n"))?;
    }

    stdout.flush()?;
    Ok(())
}

pub fn redraw<H>(
    stdout: &mut Stdout,
    app_ctx: &mut AppContext,
    start_row: u16,
    mut output_fn: H,
    n: u8,
) -> Result<(), Box<dyn Error>>
where
    H: FnMut(&mut Stdout, &Options, u8) -> Result<(), Box<dyn Error>>,
{

    queue!(
        stdout,
        cursor::MoveTo(0, start_row),
        terminal::Clear(ClearType::CurrentLine),
    )?;

    output_fn(stdout, &app_ctx.options, n)?;

    stdout.flush()?;
    Ok(())
}

pub fn redraw_selection(
    stdout: &mut Stdout,
    items: &[String],
    old: usize,
    new: usize,
    menu_start_row: u16,
) -> Result<(), Box<dyn Error>> {
    // riga old: torna “normale”
    let y_old = menu_start_row + old as u16;
    queue!(
        stdout,
        cursor::MoveTo(0, y_old),
        terminal::Clear(terminal::ClearType::CurrentLine),
        Print("   "),
        Print(&items[old])
    )?;

    // riga new: evidenziata
    let y_new = menu_start_row + new as u16;
    queue!(
        stdout,
        cursor::MoveTo(0, y_new),
        terminal::Clear(terminal::ClearType::CurrentLine),
        Print("\u{1F449} "),
        Print(&items[new])
    )?;

    stdout.flush()?;
    Ok(())
}

pub fn prompt_and_read(
  stdout: &mut Stdout,
  prompt: &str
) -> Result<String, Box<dyn Error>> {

  terminal::disable_raw_mode()?;

  let (_, rows) = terminal::size()?;

  queue!(
    stdout,
    cursor::MoveTo(0, rows - 2),                // due righe dal fondo
    crossterm::terminal::Clear(ClearType::FromCursorDown),   // cancella tutto sotto
    Print(prompt),
  )?;
  stdout.flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  terminal::enable_raw_mode()?;

  queue!(
    stdout,
    crossterm::cursor::MoveTo(0, rows - 2),
    crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
  )?;
  stdout.flush()?;

  Ok(input.trim().to_string())
}


/// Mostra una semplice dialog modale al centro del terminale con titolo e testo
pub fn show_simple_dialog(
    stdout: &mut Stdout,
    title: &str,
    items: &[&str],
    cancel_key: KeyCode,
) -> Result<usize, Box<dyn Error>> {
    // 1. prendi dimensione del terminale
    let (cols, rows) = terminal::size()?;
    let w = 50;                               // larghezza fissa
    let h = items.len() as u16 + 5;           // altezza = titolo + voci + bordi
    let x0 = (cols.saturating_sub(w)) / 2;    // colonna di partenza
    let y0 = (rows.saturating_sub(h)) / 2;    // riga di partenza

    // 2. disegno sfondo semitrasparente (opzionale)
    for y in 0..rows {
        queue!(
            stdout,
            MoveTo(0, y),
            SetBackgroundColor(Color::DarkGrey),
            Clear(ClearType::CurrentLine),
            ResetColor
        )?;
    }

    // 3. disegno bordo box
    // ┌───────┐
    queue!(
        stdout,
        SetForegroundColor(Color::White),
        MoveTo(x0, y0),
        Print("┌"),
        Print("─".repeat((w - 2) as usize)),
        Print("┐"),
    )?;
    for i in 1..h - 1 {
        queue!(
            stdout,
            MoveTo(x0, y0 + i),
            Print("│"),
            MoveTo(x0 + w - 1, y0 + i),
            Print("│"),
        )?;
    }
    // └───────┘
    queue!(
        stdout,
        MoveTo(x0, y0 + h - 1),
        Print("└"),
        Print("─".repeat((w - 2) as usize)),
        Print("┘"),
    )?;

    // 4. stampa titolo centrato
    let title_offset = x0 + (w.saturating_sub(title.len() as u16)) / 2;
    queue!(
        stdout,
        MoveTo(title_offset, y0 + 1),
        SetForegroundColor(Color::Yellow),
        Print(title),
        ResetColor
    )?;

    // 5. stampa elementi
    for (i, &item) in items.iter().enumerate() {
        queue!(
            stdout,
            MoveTo(x0 + 2, y0 + 3 + i as u16),
            SetForegroundColor(Color::White),
            SetBackgroundColor(Color::DarkGrey),
            Print(format!("{:<width$}", item, width = (w - 4) as usize)),
            ResetColor
        )?;
    }

    stdout.flush()?;

    loop {
        if let Event::Key(k) = read()? {
            if k.kind == KeyEventKind::Press /*&& ev.code == cancel_key*/ { // TODO: DA CONTROLLARE
                match k.code {
                    code if code == cancel_key => {
                        queue!(stdout, ResetColor, cursor::MoveTo(0, 0), Clear(ClearType::All), crossterm::cursor::Hide)?;
                        crossterm::terminal::enable_raw_mode()?;
                        stdout.flush()?;
                        return Ok(items.len()); // es.: len() come “annulla”
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Mostra un dialog modale al centro del terminale con titolo e voci selezionabili.
/// Ritorna l'indice scelto (0..items.len()-1).
pub fn show_dialog(
    stdout: &mut Stdout,
    title: &str,
    items: &[&str],
) -> Result<usize, Box<dyn Error>> {
    // 1. prendi dimensione del terminale
    let (cols, rows) = terminal::size()?;
    let w = 50;                               // larghezza fissa
    let h = items.len() as u16 + 4;           // altezza = titolo + voci + bordi
    let x0 = (cols.saturating_sub(w)) / 2;    // colonna di partenza
    let y0 = (rows.saturating_sub(h)) / 2;    // riga di partenza

    // 2. disegno sfondo semitrasparente (opzionale)
    for y in 0..rows {
        queue!(
            stdout,
            MoveTo(0, y),
            SetBackgroundColor(Color::DarkGrey),
            Clear(ClearType::CurrentLine),
            ResetColor
        )?;
    }

    // 3. disegno bordo box
    // ┌───────┐
    queue!(
        stdout,
        SetForegroundColor(Color::White),
        MoveTo(x0, y0),
        Print("┌"),
        Print("─".repeat((w - 2) as usize)),
        Print("┐"),
    )?;
    for i in 1..h - 1 {
        queue!(
            stdout,
            MoveTo(x0, y0 + i),
            Print("│"),
            MoveTo(x0 + w - 1, y0 + i),
            Print("│"),
        )?;
    }
    // └───────┘
    queue!(
        stdout,
        MoveTo(x0, y0 + h - 1),
        Print("└"),
        Print("─".repeat((w - 2) as usize)),
        Print("┘"),
    )?;

    // 4. stampa titolo centrato
    let title_offset = x0 + (w.saturating_sub(title.len() as u16)) / 2;
    queue!(
        stdout,
        MoveTo(title_offset, y0 + 1),
        SetForegroundColor(Color::Yellow),
        Print(title),
        ResetColor
    )?;

    // 5. ciclo di input per navigare voci
    let mut selected = 0;
    loop {
        // ridisegna le voci con evidenza
        for (i, &item) in items.iter().enumerate() {
            let fg = if i == selected { Color::Black } else { Color::White };
            let bg = if i == selected { Color::White } else { Color::DarkGrey };
            queue!(
                stdout,
                MoveTo(x0 + 2, y0 + 2 + i as u16),
                SetForegroundColor(fg),
                SetBackgroundColor(bg),
                Print(format!("{:<width$}", item, width = (w - 4) as usize)),
                ResetColor
            )?;
        }

        stdout.flush()?;

        // leggi tasto
        if let Event::Key(k) = read()? {
            match k.code {
                KeyCode::Up if selected > 0 => selected -= 1,
                KeyCode::Down if selected + 1 < items.len() => selected += 1,
                KeyCode::Enter => {
                    // prima di tornare, ripristina lo schermo (opzionale)
                    terminal::disable_raw_mode()?;
                    queue!(stdout, ResetColor, Clear(ClearType::All))?;
                    stdout.flush()?;
                    return Ok(selected);
                }
                KeyCode::Esc => {
                    terminal::disable_raw_mode()?;
                    queue!(stdout, ResetColor, Clear(ClearType::All))?;
                    stdout.flush()?;
                    return Ok(items.len()); // es.: len() come “annulla”
                }
                _ => {}
            }
        }
    }
}

/// Restituisce una `String` con percentuale e colore graduato.
/// - Se v > 0: sfuma da (rrr,ggg,bbb) fino a (rrr,ggg,bbb)
/// - Se v < 0: sfuma da (rrr,ggg,bbb) fino a (rrr,ggg,bbb)
pub fn shade_percent(v: f32, max_range: f32) -> String {
    // Limitiamo il valore tra -max_range e +max_range
    //let v = v.clamp(-max_range, max_range);
    let ratio = (v.abs() / max_range).clamp(0.0, 1.0);

    // Definiamo start/end color per segno
    let (r0, g0, b0, r1, g1, b1) = if v >= 0.0 {
        // verde scuro → verde chiaro
        (0, 100, 0, 144, 238, 144)
    } else {
        // rosso scuro → rosso vivace
        //(139, 0, 0, 255, 0, 0)
        // rosso scuro → arancione
        (139, 0, 0, 255, 165, 0)
    };

    // Interpoliamo i canali
    let r = (r0 as f32 + (r1 as f32 - r0 as f32) * ratio) as u8;
    let g = (g0 as f32 + (g1 as f32 - g0 as f32) * ratio) as u8;
    let b = (b0 as f32 + (b1 as f32 - b0 as f32) * ratio) as u8;

    format!("{:^16}", format!("{:+}%", v))
        .truecolor(r, g, b)
        .to_string()
}

pub fn shade_fixed(v: f32) -> String { // TODO:
    if v > 1.0 {
        //format!("{:^16}", format!("{:+}%", v)).truecolor(144, 238, 144) // verde chiaro (doesn't work on Win: gray)
        format!("{:^16}", format!("{:+}%", v)).truecolor(0, 255, 0)
    } else if v > 0.5 {
        format!("{:^16}", format!("{:+}%", v)).truecolor(0, 200, 0)  // verde medio
    } else if v > 0.0 {
        format!("{:^16}", format!("{:+}%", v)).truecolor(0, 128, 0)
    } else if v < -1.0 {
        //format!("{:^16}", format!("{:+}%", v)).truecolor(255,165,0)   // arancione
        format!("{:^16}", format!("{:+}%", v)).truecolor(255,100,0)
    } else if v < -0.5 {
        format!("{:^16}", format!("{:+}%", v)).truecolor(255, 0, 0)   // rosso
    } else if v < 0.0 {
        format!("{:^16}", format!("{:+}%", v)).truecolor(139, 0, 0)   // rosso scuro
    } else {
        format!("{:^16}", "").yellow()               // zero “giallo”
    }.to_string()
}