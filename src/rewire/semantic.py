"""Optional semantic search: offline static embeddings (WordLlama, weights ship in the wheel).

Enabled when `pip install 'rewire[semantic]'` is present; rewire works without it.
No server, no API, no download at run time: ~0.2 s to load, ~10 µs per text.
"""

import os
import shutil
from functools import lru_cache

DIM = 256


@lru_cache(maxsize=1)
def model():
    """The embedding model, or None when the optional dependency is missing."""
    if os.environ.get("REWIRE_SEMANTIC") == "0":
        return None
    try:
        import wordllama
        from wordllama import WordLlama
    except ImportError:
        return None
    try:
        from pathlib import Path

        # wordllama looks for the bundled tokenizer in "tokenizer/" but ships it in "tokenizers/";
        # copy it into the cache it checks so no network access is ever needed.
        name = "l2_supercat_tokenizer_config.json"
        src = Path(wordllama.__file__).parent / "tokenizers" / name
        dst = WordLlama.get_file_path("tokenizer", None) / name
        if src.exists() and not dst.exists():
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(src, dst)
        return WordLlama.load(dim=DIM, disable_download=True)
    except Exception:  # broken install: degrade to keyword search
        return None


def available() -> bool:
    return model() is not None


def embed(texts: list[str]):
    """L2-normalised float32 matrix (len(texts) x DIM), or None if unavailable."""
    m = model()
    if m is None or not texts:
        return None
    import numpy as np

    vecs = np.asarray(m.embed(texts), dtype=np.float32)
    norms = np.linalg.norm(vecs, axis=1, keepdims=True)
    norms[norms == 0] = 1.0
    return vecs / norms
