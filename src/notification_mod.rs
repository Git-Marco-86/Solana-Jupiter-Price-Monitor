/**
 * v1.2.0 26/06/2025
 * Author: Marco Maffei
 * 
 */

use std::io;
use std::process::Command;
use std::path::PathBuf;

pub fn send_notification(title: &str, message: &str) -> io::Result<()> {
  let command = format!(
      "Import-Module BurntToast; New-BurntToastNotification -Text \"{}\", \"{}\"",
      title, message
  );

  #[cfg(windows)]
  let ps = PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
  #[cfg(not(windows))]
  let ps = PathBuf::from("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe");

  Command::new(ps)
      .args(&["-ExecutionPolicy", "Bypass", "-Command", &command])
      .status()?;

  Ok(())
}