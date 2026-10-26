//! Errores que se muestran al usuario como `  [ERROR] <mensaje>`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmhError {
    /// Equivalente a `fail(msg)` de la version Python: el mensaje se muestra tal cual.
    #[error("{0}")]
    Fail(String),
}

/// Crea el error de `fail(msg)`.
pub fn fail(msg: impl Into<String>) -> anyhow::Error {
    OmhError::Fail(msg.into()).into()
}

/// Texto que se muestra tras `[ERROR] `. Si en la cadena hay un error propio de omh
/// se muestra solo ese, como hacia Python; si no, el error con su contexto.
pub fn user_message(err: &anyhow::Error) -> String {
    match err.chain().find_map(|e| e.downcast_ref::<OmhError>()) {
        Some(own) => own.to_string(),
        None => format!("{err:#}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn muestra_el_error_propio_aunque_tenga_contexto() {
        let err = Err::<(), _>(fail("Falta el paso 2."))
            .context("al cargar")
            .unwrap_err();
        assert_eq!(user_message(&err), "Falta el paso 2.");
    }

    #[test]
    fn otros_errores_muestran_el_contexto() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no existe");
        let err = Err::<(), _>(io).context("No se pudo leer x").unwrap_err();
        assert_eq!(user_message(&err), "No se pudo leer x: no existe");
    }
}
