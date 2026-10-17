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


# Experimental embedding models for A/B runs (REWIRE_EMBED); the default stays WordLlama. Each brain's index must be
# built with one model: vectors of different models are not comparable.
EMBED = os.environ.get("REWIRE_EMBED", "wordllama")


class _Potion:  # Model2Vec static embeddings (minishlab/potion-base-8M), offline after the first download
    def __init__(self):
        from model2vec import StaticModel
        self.m = StaticModel.from_pretrained("minishlab/potion-base-8M")

    def embed(self, texts):
        return self.m.encode(texts)


class _Bge:  # BAAI/bge-small-en-v1.5 through fastembed (ONNX, CPU); queries get bge's search instruction
    def __init__(self):
        from fastembed import TextEmbedding
        self.m = TextEmbedding("BAAI/bge-small-en-v1.5")

    def embed(self, texts):
        return list(self.m.embed(texts))

    def embed_query(self, texts):
        return list(self.m.query_embed(texts))


@lru_cache(maxsize=1)
def _alt_model():
    try:
        return {"potion": _Potion, "bge-small": _Bge}[EMBED]()
    except Exception:  # missing package or model: degrade to keyword search, as for WordLlama
        return None


def _active():
    return model() if EMBED == "wordllama" else _alt_model()


def available() -> bool:
    return _active() is not None


def embed_query(texts: list[str]):
    """Like embed(), for search queries (models such as bge embed queries differently from passages)."""
    m = _active()
    if m is not None and hasattr(m, "embed_query") and texts:
        return _normalise(m.embed_query(texts))
    return embed(texts)


def embed(texts: list[str]):
    """L2-normalised float32 matrix (len(texts) x DIM), or None if unavailable."""
    m = _active()
    if m is None or not texts:
        return None
    return _normalise(m.embed(texts))


def _normalise(raw):
    import numpy as np

    vecs = np.asarray(raw, dtype=np.float32)
    norms = np.linalg.norm(vecs, axis=1, keepdims=True)
    norms[norms == 0] = 1.0
    return vecs / norms


# Experimental cross-encoder reranking of excerpt candidates for A/B runs (REWIRE_RERANK=<fastembed model>).
RERANK = os.environ.get("REWIRE_RERANK", "")
RERANK_POOL = int(os.environ.get("REWIRE_RERANK_POOL", "30"))


@lru_cache(maxsize=1)
def _reranker():
    if not RERANK:
        return None
    try:
        from fastembed.rerank.cross_encoder import TextCrossEncoder
        return TextCrossEncoder(RERANK)
    except Exception:
        return None


def rerank(query: str, texts: list[str]) -> list[float] | None:
    """Cross-encoder relevance of each text to the query, or None when no reranker is configured."""
    r = _reranker()
    if r is None or not texts:
        return None
    return [float(x) for x in r.rerank(query, texts)]
