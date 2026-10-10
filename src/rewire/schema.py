"""MVP ontology: 7 node types, 5 relations (docs/design/mvp-spec.md §3)."""

CLAIM_TYPES = ("constraint", "preference", "fact", "decision")
HUB_TYPES = ("project", "person", "concept")
TYPES = CLAIM_TYPES + HUB_TYPES
RELATIONS = ("about", "part_of", "supersedes", "contradicts", "related_to")
DECAY_EXEMPT = ("constraint", "preference")
CONFIDENCE = ("explicit", "inferred")
PROVENANCE = ("user", "assistant", "tool")

REQUIRED = ("id", "type", "title", "scope", "importance", "confidence", "provenance", "recorded_at")


class SchemaError(ValueError):
    pass


def validate(meta: dict, body: str) -> None:
    for key in REQUIRED:
        if meta.get(key) in (None, ""):
            raise SchemaError(f"missing field: {key}")
    if meta["type"] not in TYPES:
        raise SchemaError(f"unknown type: {meta['type']}")
    if meta["confidence"] not in CONFIDENCE:
        raise SchemaError(f"bad confidence: {meta['confidence']}")
    if meta["provenance"] not in PROVENANCE:
        raise SchemaError(f"bad provenance: {meta['provenance']}")
    if not isinstance(meta["importance"], int) or not 1 <= meta["importance"] <= 5:
        raise SchemaError("importance must be an int 1-5")
    scope = meta["scope"]
    if scope != "global" and not (isinstance(scope, str) and scope.startswith("project:") and len(scope) > 8):
        raise SchemaError(f"bad scope: {scope}")
    for edge in meta.get("edges") or []:
        if not isinstance(edge, dict) or edge.get("rel") not in RELATIONS or not edge.get("to"):
            raise SchemaError(f"bad edge: {edge}")
    if meta["type"] in CLAIM_TYPES and len(body.strip()) > 600:
        raise SchemaError("claim body too long; claims must be atomic (<= 600 chars)")
    if not meta["title"].strip() or len(meta["title"]) > 160:
        raise SchemaError("title must be 1-160 chars")
