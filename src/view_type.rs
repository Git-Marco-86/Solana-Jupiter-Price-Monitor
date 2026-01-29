/**
 * v1.0.0 01/07/2025
 * Author: Marco Maffei
 *
 */

use serde::{Serialize, Deserialize};

// let id: u8 = app_ctx.options.view_type.id();
// let s: &'static str = app_ctx.options.view_type.as_str();

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum ViewType {
    #[serde(alias = "sequenziale")]
    SEQUENTIAL,
    #[serde(alias = "fissa")]
    FIXED,
}

impl ViewType {
    /// Ritorna il simbolo come stringa
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewType::SEQUENTIAL => "sequential",
            ViewType::FIXED => "fixed",
        }
    }

    pub fn label(&self, locale: &str) -> &'static str {
        match (self, locale) {
            // SEQUENTIAL
            (ViewType::SEQUENTIAL, "it") => "Sequenziale",
            (ViewType::SEQUENTIAL, _)    => "Sequential",
            // FIXED
            (ViewType::FIXED,      "it") => "Fissa",
            (ViewType::FIXED,      _)    => "Fixed",
        }
    }

    /// Un id associato a ciascuna variante.
    #[allow(dead_code)]
    pub fn id(&self) -> u8 {
        match self {
            ViewType::SEQUENTIAL => 1,
            ViewType::FIXED => 2,
        }
    }

    /// Restituisce tutte le varianti dell'enum come slice.
    pub fn variants() -> &'static [ViewType] {
        &[ViewType::SEQUENTIAL, ViewType::FIXED]
    }

    /// Converte un id in una variante, se possibile.
    #[allow(dead_code)]
    pub fn from_id(id: u8) -> Option<ViewType> {
        match id {
            1 => Some(ViewType::SEQUENTIAL),
            2 => Some(ViewType::FIXED),
            _ => None,
        }
    }

    /// Restituisce la variante successiva ciclicamente
    pub fn next(self) -> ViewType {
        // Prendiamo il slice statico con tutte le varianti
        let vs = ViewType::variants();
        // Troviamo l’indice di self
        let idx = vs.iter()
            .position(|&v| v == self)
            .expect("Self non trovato in variants()");
        // Restituiamo la successiva, modulare sul numero di varianti
        vs[(idx + 1) % vs.len()]
    }
}