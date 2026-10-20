//! Optional semantic search: offline static embeddings (WordLlama l2_supercat, 256 dims). A vector is the
//! mean of the token embeddings, L2-normalised.
//!
//! The model files are not part of the binary: `rewire semantic install` copies them from a local
//! `wordllama` Python package (or a directory given with --from) into the model directory. Without
//! them, or with REWIRE_SEMANTIC=0, recall is keyword-only. No server, no API, no network at run time.

use std::path::PathBuf;

pub const DIM: usize = 256;
pub const WEIGHTS: &str = "l2_supercat_256.safetensors";
pub const TOKENIZER: &str = "l2_supercat_tokenizer_config.json";

/// $REWIRE_MODEL_DIR, else $XDG_DATA_HOME/rewire/wordllama (~/.local/share/rewire/wordllama).
pub fn model_dir() -> PathBuf {
    if let Ok(d) = std::env::var("REWIRE_MODEL_DIR") {
        if !d.is_empty() {
            return crate::brain::expand_user(&d);
        }
    }
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::brain::expand_user("~/.local/share"));
    base.join("rewire").join("wordllama")
}

#[cfg(feature = "semantic")]
mod imp {
    use super::*;
    use std::sync::OnceLock;

    pub struct Model {
        /// f16 rows, vocab x DIM, little endian
        weights: Vec<u8>,
        vocab: usize,
        tokenizer: tokenizers::Tokenizer,
    }

    fn f16_to_f32(bits: u16) -> f32 {
        let sign = ((bits >> 15) as u32) << 31;
        let exp = ((bits >> 10) & 0x1f) as u32;
        let frac = (bits & 0x3ff) as u32;
        let out = match (exp, frac) {
            (0, 0) => sign,
            (0, _) => {
                // subnormal: normalise
                let mut e = 127 - 15 + 1;
                let mut f = frac;
                while f & 0x400 == 0 {
                    f <<= 1;
                    e -= 1;
                }
                sign | ((e as u32) << 23) | ((f & 0x3ff) << 13)
            }
            (0x1f, _) => sign | 0x7f80_0000 | (frac << 13),
            _ => sign | ((exp + 127 - 15) << 23) | (frac << 13),
        };
        f32::from_bits(out)
    }

    fn load() -> Option<Model> {
        if std::env::var("REWIRE_SEMANTIC").as_deref() == Ok("0") {
            return None;
        }
        let dir = model_dir();
        let raw = std::fs::read(dir.join(WEIGHTS)).ok()?;
        // safetensors: u64 header length, JSON header, then the tensor bytes
        let n = u64::from_le_bytes(raw.get(..8)?.try_into().ok()?) as usize;
        let header: serde_json::Value = serde_json::from_slice(raw.get(8..8 + n)?).ok()?;
        let t = header.get("embedding.weight")?;
        if t.get("dtype")?.as_str()? != "F16" {
            return None;
        }
        let shape: Vec<usize> = t.get("shape")?.as_array()?.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect();
        let offs: Vec<usize> = t.get("data_offsets")?.as_array()?.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect();
        if shape.len() != 2 || shape[1] != DIM || offs.len() != 2 || offs[1] - offs[0] != shape[0] * DIM * 2 {
            return None;
        }
        let weights = raw.get(8 + n + offs[0]..8 + n + offs[1])?.to_vec();
        let tokenizer = tokenizers::Tokenizer::from_file(dir.join(TOKENIZER)).ok()?;
        Some(Model { weights, vocab: shape[0], tokenizer })
    }

    pub fn model() -> Option<&'static Model> {
        static M: OnceLock<Option<Model>> = OnceLock::new();
        M.get_or_init(load).as_ref()
    }

    impl Model {
        fn embed_one(&self, text: &str) -> Vec<f32> {
            let ids: Vec<u32> = self.tokenizer.encode(text, false).map(|e| e.get_ids().to_vec()).unwrap_or_default();
            let mut v = vec![0f32; DIM];
            for &id in &ids {
                let row = (id as usize).min(self.vocab - 1) * DIM * 2;
                for (j, x) in v.iter_mut().enumerate() {
                    let b = &self.weights[row + 2 * j..row + 2 * j + 2];
                    *x += f16_to_f32(u16::from_le_bytes([b[0], b[1]]));
                }
            }
            let count = (ids.len() as f32).max(1.0);
            for x in v.iter_mut() {
                *x /= count;
            }
            let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm = if norm == 0.0 { 1.0 } else { norm };
            v.iter().map(|x| x / norm).collect()
        }
    }

    /// Cheap check (the index asks on every open): the files are there. Loading the model (~70 ms,
    /// mostly the tokenizer) happens on the first embed only, so `prime` never pays for it.
    pub fn available() -> bool {
        static A: OnceLock<bool> = OnceLock::new();
        *A.get_or_init(|| {
            std::env::var("REWIRE_SEMANTIC").as_deref() != Ok("0")
                && model_dir().join(WEIGHTS).is_file()
                && model_dir().join(TOKENIZER).is_file()
        })
    }

    pub fn embed(texts: &[String]) -> Option<Vec<Vec<f32>>> {
        let m = model()?;
        Some(texts.iter().map(|t| m.embed_one(t)).collect())
    }
}

#[cfg(not(feature = "semantic"))]
mod imp {
    pub fn available() -> bool {
        false
    }
    pub fn embed(_texts: &[String]) -> Option<Vec<Vec<f32>>> {
        None
    }
}

pub use imp::{available, embed};

pub fn to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub fn from_blob(b: &[u8]) -> Vec<f32> {
    b.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Copy the model files from `from` (a directory holding them, or a wordllama package directory with
/// weights/ and tokenizers/) or, without it, from the `wordllama` package of a local Python.
pub fn install(from: Option<&str>) -> crate::Result<String> {
    let src = match from {
        Some(d) => crate::brain::expand_user(d),
        None => {
            let out = std::process::Command::new("python3")
                .args(["-c", "import os, wordllama; print(os.path.dirname(wordllama.__file__))"])
                .output()
                .map_err(|e| crate::Error::Op(format!("python3 not found ({e}); use --from <dir>")))?;
            let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !out.status.success() || dir.is_empty() {
                return Err(crate::Error::Op(
                    "no wordllama package found: `pip install wordllama`, or pass --from <dir with the model files>".into(),
                ));
            }
            PathBuf::from(dir)
        }
    };
    let find =
        |name: &str| [src.join(name), src.join("weights").join(name), src.join("tokenizers").join(name)].into_iter().find(|p| p.is_file());
    let (Some(w), Some(t)) = (find(WEIGHTS), find(TOKENIZER)) else {
        return Err(crate::Error::Op(format!("{WEIGHTS} and {TOKENIZER} not found under {}", src.display())));
    };
    let dst = model_dir();
    std::fs::create_dir_all(&dst)?;
    std::fs::copy(w, dst.join(WEIGHTS))?;
    std::fs::copy(t, dst.join(TOKENIZER))?;
    Ok(format!("semantic model installed in {} (the index is rebuilt on next use)", dst.display()))
}
