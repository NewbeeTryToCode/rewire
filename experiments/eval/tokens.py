"""Token counts for context and prompts: the GPT-4o tokenizer (o200k) when it can be loaded, else characters / 4.

The tokenizer file is read from $REWIRE_TOKENIZER, or downloaded once from Hugging Face (Xenova/gpt-4o) into
~/.cache/rewire-eval/. Answer and judge calls also report the provider's own (billed) prompt token counts; those
are the exact numbers for the model used (Claude tokenizes differently from GPT-4o).
"""

import math
import os
import threading
import urllib.request

URL = "https://huggingface.co/Xenova/gpt-4o/resolve/main/tokenizer.json"
_lock = threading.Lock()
_tok = None
_name = None


def _load():
    global _tok, _name
    with _lock:
        if _name is not None:
            return
        path = os.environ.get("REWIRE_TOKENIZER") or os.path.expanduser("~/.cache/rewire-eval/gpt-4o-tokenizer.json")
        try:
            from tokenizers import Tokenizer

            if not os.path.exists(path):
                os.makedirs(os.path.dirname(path), exist_ok=True)
                urllib.request.urlretrieve(URL, path + ".part")
                os.replace(path + ".part", path)
            _tok, _name = Tokenizer.from_file(path), "gpt-4o (o200k)"
        except Exception:  # no tokenizers package or no network: estimate
            _tok, _name = None, "chars/4 estimate"


def name() -> str:
    _load()
    return _name


def count(text: str) -> int:
    _load()
    if _tok is None:
        return math.ceil(len(text) / 4)
    return len(_tok.encode(text, add_special_tokens=False).ids)
