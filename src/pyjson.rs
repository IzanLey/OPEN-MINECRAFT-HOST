//! JSON con exactamente el mismo formato que `json.dump` de Python.
//!
//! - `to_pretty`: `json.dump(obj, indent=2)` (separadores `,` y `: `).
//! - `to_compact`: `json.dump(obj)` (separadores `, ` y `: `).
//!
//! En los dos casos todo lo que no es ASCII imprimible se escribe como `\uXXXX`
//! en minusculas (`ensure_ascii=True`), con pares sustitutos fuera del plano basico.

use std::io::{self, Write};

use serde::Serialize;
use serde_json::ser::{Formatter, Serializer};

pub fn to_pretty<T: Serialize + ?Sized>(value: &T) -> anyhow::Result<String> {
    to_string(value, true)
}

pub fn to_compact<T: Serialize + ?Sized>(value: &T) -> anyhow::Result<String> {
    to_string(value, false)
}

fn to_string<T: Serialize + ?Sized>(value: &T, pretty: bool) -> anyhow::Result<String> {
    let mut out = Vec::new();
    let formatter = PyFormatter {
        pretty,
        depth: 0,
        has_value: false,
    };
    value.serialize(&mut Serializer::with_formatter(&mut out, formatter))?;
    Ok(String::from_utf8(out)?)
}

struct PyFormatter {
    pretty: bool,
    depth: usize,
    has_value: bool,
}

impl PyFormatter {
    fn open<W: ?Sized + Write>(&mut self, w: &mut W, bracket: &[u8]) -> io::Result<()> {
        self.depth += 1;
        self.has_value = false;
        w.write_all(bracket)
    }

    fn close<W: ?Sized + Write>(&mut self, w: &mut W, bracket: &[u8]) -> io::Result<()> {
        self.depth = self.depth.saturating_sub(1);
        if self.pretty && self.has_value {
            self.newline(w)?;
        }
        w.write_all(bracket)
    }

    fn item<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        if self.pretty {
            if !first {
                w.write_all(b",")?;
            }
            self.newline(w)
        } else if first {
            Ok(())
        } else {
            w.write_all(b", ")
        }
    }

    fn newline<W: ?Sized + Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(b"\n")?;
        for _ in 0..self.depth {
            w.write_all(b"  ")?;
        }
        Ok(())
    }
}

impl Formatter for PyFormatter {
    fn begin_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.open(w, b"[")
    }

    fn end_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.close(w, b"]")
    }

    fn begin_array_value<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.item(w, first)
    }

    fn end_array_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    fn begin_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.open(w, b"{")
    }

    fn end_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.close(w, b"}")
    }

    fn begin_object_key<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.item(w, first)
    }

    fn begin_object_value<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        w.write_all(b": ")
    }

    fn end_object_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    fn write_string_fragment<W: ?Sized + Write>(
        &mut self,
        w: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        // serde_json ya escapa comillas, barras invertidas y controles; aqui llega el
        // resto, y Python escapa todo lo que no esta entre ' ' y '~'.
        for c in fragment.chars() {
            if (' '..='~').contains(&c) {
                w.write_all(&[c as u8])?;
            } else {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    write!(w, "\\u{unit:04x}")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> serde_json::Value {
        json!({
            "name": "Se\u{f1}or", "nonbmp": "\u{1D11E}", "ctrl": "a\u{1}b\u{7f}\t\n\"\\/\r\u{8}\u{c}",
            "list": [], "obj": {}, "nested": {"a": [1, 2, {"b": null}]}, "t": true, "f": false, "n": -5
        })
    }

    // Salidas de json.dumps de Python con el mismo valor.
    #[test]
    fn igual_que_json_dump_con_indent_2() {
        let expected = "{\n  \"name\": \"Se\\u00f1or\",\n  \"nonbmp\": \"\\ud834\\udd1e\",\n  \"ctrl\": \"a\\u0001b\\u007f\\t\\n\\\"\\\\/\\r\\b\\f\",\n  \"list\": [],\n  \"obj\": {},\n  \"nested\": {\n    \"a\": [\n      1,\n      2,\n      {\n        \"b\": null\n      }\n    ]\n  },\n  \"t\": true,\n  \"f\": false,\n  \"n\": -5\n}";
        assert_eq!(to_pretty(&sample()).unwrap(), expected);
    }

    #[test]
    fn igual_que_json_dump_compacto() {
        let expected = "{\"name\": \"Se\\u00f1or\", \"nonbmp\": \"\\ud834\\udd1e\", \"ctrl\": \"a\\u0001b\\u007f\\t\\n\\\"\\\\/\\r\\b\\f\", \"list\": [], \"obj\": {}, \"nested\": {\"a\": [1, 2, {\"b\": null}]}, \"t\": true, \"f\": false, \"n\": -5}";
        assert_eq!(to_compact(&sample()).unwrap(), expected);
    }

    #[test]
    fn run_json_compacto() {
        let run = json!({"port": 51234, "token": "00ff", "pid": 1, "wrapper_pid": 2, "started": "2026-10-26 00:05:51"});
        assert_eq!(
            to_compact(&run).unwrap(),
            "{\"port\": 51234, \"token\": \"00ff\", \"pid\": 1, \"wrapper_pid\": 2, \"started\": \"2026-10-26 00:05:51\"}"
        );
    }

    #[test]
    fn listas_y_objetos_vacios_anidados() {
        assert_eq!(to_pretty(&json!([])).unwrap(), "[]");
        assert_eq!(to_pretty(&json!({"a": []})).unwrap(), "{\n  \"a\": []\n}");
        assert_eq!(to_pretty(&json!([[], {}])).unwrap(), "[\n  [],\n  {}\n]");
    }
}
