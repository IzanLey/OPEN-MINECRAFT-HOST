//! Errores que se muestran al usuario como `  [ERROR] <mensaje>`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmhError {
    /// Equivalente a `fail(msg)` de la version Python: el mensaje se muestra tal cual.
    #[error("{0}")]
    Fail(String),
    /// Respuesta HTTP que no es 2xx (`urllib.error.HTTPError`).
    #[error("El servidor remoto respondio {code} ({url}).")]
    Http { code: u16, url: String },
    /// No se pudo conectar o se corto la conexion (`urllib.error.URLError`).
    #[error("Error de red: {0}. Comprueba tu conexion a internet.")]
    Network(String),
}

/// Crea el error de `fail(msg)`.
pub fn fail(msg: impl Into<String>) -> anyhow::Error {
    OmhError::Fail(msg.into()).into()
}

/// Codigo HTTP si el error es una respuesta de error de un servidor.
pub fn http_status(err: &anyhow::Error) -> Option<u16> {
    err.chain()
        .find_map(|e| match e.downcast_ref::<OmhError>() {
            Some(OmhError::Http { code, .. }) => Some(*code),
            _ => None,
        })
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
    fn errores_http_y_de_red_como_python() {
        let err: anyhow::Error = OmhError::Http {
            code: 404,
            url: "https://x/y".into(),
        }
        .into();
        assert_eq!(
            user_message(&err),
            "El servidor remoto respondio 404 (https://x/y)."
        );
        assert_eq!(http_status(&err), Some(404));
        let err: anyhow::Error = OmhError::Network("sin conexion".into()).into();
        assert_eq!(
            user_message(&err),
            "Error de red: sin conexion. Comprueba tu conexion a internet."
        );
        assert_eq!(http_status(&err), None);
    }

    #[test]
    fn otros_errores_muestran_el_contexto() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no existe");
        let err = Err::<(), _>(io).context("No se pudo leer x").unwrap_err();
        assert_eq!(user_message(&err), "No se pudo leer x: no existe");
    }
}
