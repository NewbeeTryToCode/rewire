# Local storage and retrieval backends for a per-user agent memory (as of October 2026)

Context (read from the repo, not modified): rewire keeps the source of truth in markdown and JSONL in git. `src/rewire/index.py` builds a rebuildable SQLite index with FTS5 (`unicode61 remove_diacritics 2`, BM25 with column weights 3/1/2, ties broken by id/path/turn "identical in every implementation"). Vectors are 256-d float32 BLOBs in plain tables (`vec`, `epvec`). They are loaded into a numpy matrix and scanned brute force (`mat @ qv`, then a full `argsort`). Memories fuse BM25 magnitudes with a rank-decayed semantic bonus (SEM_WEIGHT 0.15, top 10). Episodes use weighted RRF (k=60, semantic weight 0.25). The embeddings are static WordLlama 256-d (`semantic.py`: "~0.2 s to load, ~10 µs per text", no network). The repo's own measurements in `docs/evaluation.md` and `index.py` comments show the following:
- The static vectors alone are much weaker than BM25 + keywords: R@5 57% vs 90%.
- At equal RRF weight, the static vectors hurt LongMemEval single-session-user all-evidence (100% → 95%).
- At weight 0.25 they help preference questions (77% → 87%).
- Retrieval latency is p50 4 ms / p99 16 ms on LoCoMo, and p50 21 ms / p99 195 ms on LongMemEval_S (all 500).
- The repo's own "next step" for multi-hop is "a small contextual encoder or a cross-encoder".

**Own measurements made for this note (2026-10-17).** These are labelled "measured here" below.
- Hardware: a cloud container with 4 vCPU, Intel Xeon @ 2.80 GHz and 15 GB RAM. This is not a laptop. A recent laptop (Apple M-series, or a Zen4 or Intel 12th-gen or later) is probably at least as fast per core, but that was not verified.
- Vectors: random float32 data, L2-normalised for the sqlite-vec runs. Timings are the mean of 5 queries after one warm-up query.
- Scripts: `/tmp/claude-0/-home-user-rewire/e7c4f66f-47f5-5774-a842-5d3677ee5076/scratchpad/bvec.py` (sqlite-vec) and an inline numpy script. The scratchpad is ephemeral.

## 1. SQLite FTS5 + sqlite-vec (or sqlite-vss): vector performance at 10^4–10^6, quantization, ANN, bindings, maturity

### Takeaway
sqlite-vec is still a brute-force scanner. The PyPI package is at 0.1.9, dual MIT/Apache-2.0, and has no confirmed ANN index. In this container its float32 `vec0` scan was about 10× slower than rewire's numpy matmul: 52 ms at 100k×256-d and 526 ms at 1M×256-d. Binary quantization brings it to about 16 ms at 100k but is approximate. For rewire it adds little over the current BLOB + numpy design, except SQL-side filtering and a single-file Rust/Python-portable scan.

### Cited Findings
- sqlite-vec scans brute force and has no ANN index, so recall is exact. A secondhand guide reports about 17 ms for k=20 on SIFT1M (1M × 128-d) on an M1 Mac, against about 136 ms for NumPy and 10 ms for FAISS. No methodology is given, so treat it as unverified. — [Mixpeek guide](https://mixpeek.com/guides/do-you-need-a-vector-database)
- Marco Bambini measured 100,000 × 384-d float32 vectors on an M1 Pro: a full sqlite-vec scan took 67.84 ms, and quantized variants of other SQLite vector extensions were much faster. I saw this only as a search snippet; the page itself could not be fetched (DNS blocked). — [The State of Vector Search in SQLite](https://marcobambini.substack.com/p/the-state-of-vector-search-in-sqlite)
- The vectorlite project (an HNSW-based SQLite extension) states that sqlite-vec "only uses brute-force, which doesn't scale with large datasets". Its own benchmarks go only up to 20k vectors. — [vectorlite GitHub](https://github.com/1yefuwang1/vectorlite)
- **Measured here: sqlite-vec 0.1.9** (PyPI, licence metadata "MIT License, Apache License 2.0"), `vec0` virtual table, `distance_metric=cosine`, k=20. On every run the top-1 result was the query vector itself, so it was exact. A plain `vec_distance_hamming` scan over `vec_quantize_binary` BLOBs gave the binary-quantized numbers.

  | dim | n | float32 k=20 | binary-quantized scan |
  |---|---|---|---|
  | 256 | 10,000 | 5.0 ms | 1.4 ms |
  | 256 | 100,000 | 52.5 ms | 16.1 ms |
  | 256 | 1,000,000 | 525.9 ms | 192.3 ms |
  | 768 | 100,000 | 171.6 ms | 17.6 ms |
  | 3072 | 100,000 | 643.0 ms | 35.6 ms |

  Source: own run, scratchpad `bvec.py`.
- sqlite-vss (Faiss-based, by the same author) is the predecessor to sqlite-vec. I found no 2026 source showing active development. — no primary source fetched; see Gaps.
- I found no sqlite-vec release with DiskANN or another ANN index in 2026 search results. DiskANN news in 2026 concerned Azure SQL (GA September 2026), SurrealDB 3.1 and Zvec. — [Azure SQL DiskANN GA](https://devblogs.microsoft.com/azure-sql/diskann-vector-index-search-are-now-generally-available-in-azure-sql/); [SurrealDB 3.1](https://surrealdb.com/blog/surrealdb-3-1-stability-diskann-and-a-new-release-process.md); [Zvec DiskANN](https://zvec.org/en/blog/2026-08-04-zvec-diskann/)

### Inferences
- sqlite-vec's float32 scan was about 10× slower than numpy's BLAS matvec on the same box: 52 ms vs 4 ms at 100k×256. One likely cause is that the generic wheel lacks AVX-tuned kernels. This is consistent with the secondhand M1 number, where FAISS was faster than sqlite-vec. For rewire, moving vectors into `vec0` would make latency worse in Python. Its benefits would be:
  - one C kernel that Python and Rust both call, which helps determinism (see §5);
  - metadata/partition filtering in SQL, e.g. by `scope`.
- Binary quantization at 256-d (32 bytes per vector) is likely too lossy for an already-weak static model. The standard pattern is to oversample binary candidates and then rescore with float32. That pattern is untested here.
- FTS5 itself is not the bottleneck: the repo reports LoCoMo retrieval at p50 4 ms.

### Gaps
- sqlite-vec's GitHub releases and roadmap (ANN plans, 0.2.x) could not be checked: the GitHub API was not enabled for this session and alexgarcia.xyz did not resolve. Whether a newer version than 0.1.9 exists on crates.io was also not checked.
- No laptop (Apple M-series) numbers for sqlite-vec were measured.

## 2. LanceDB, DuckDB VSS, Chroma, Qdrant (embedded), Postgres + pgvector/pgvectorscale

### Takeaway
The embedded engines that give BM25 + vector hybrid in-process are LanceDB, Qdrant Edge and Chroma. Each brings its own on-disk format, its own BM25 implementation and ANN non-determinism. None would score BM25 identically to SQLite FTS5, so the Python/Rust parity rewire has today would have to be re-established. DuckDB VSS still marks HNSW persistence as experimental. Postgres options are server processes, which is overkill for a per-user local index of at most 10^6 items.

### Cited Findings
- **LanceDB.** It describes itself as "an open source, embedded and developer-friendly vector database".
  - The docs cite 26 ms full-text-search query latency (vendor, setup not stated in the snippet).
  - The September 2025 changelog says FTS latencies were "reduced significantly".
  - The enterprise page reports about 25 ms P50 for warm vector search and about 50 ms P99 for filtered search. That is the cloud/enterprise product, flagged by the vendor as "representative… not universal guarantees".
  
  — [LanceDB FTS docs](https://docs.lancedb.com/core/full-text-search); [LanceDB changelog](https://lancedb.com/docs/changelog); [LanceDB enterprise benchmarks](https://docs.lancedb.com/enterprise/benchmarks); [lancedb-study (prrao87)](https://github.com/prrao87/lancedb-study)
- LanceDB has an older Tantivy-based FTS path alongside native FTS. — [LanceDB FTS with Tantivy](https://www.lancedb.com/docs/search/full-text-search-tantivy)
- **DuckDB VSS (HNSW).**
  - By default, HNSW indexes can only be created in in-memory databases unless `hnsw_enable_experimental_persistence` is set.
  - WAL recovery is not implemented for custom indexes, so a crash with uncommitted changes can corrupt the index or lose data.
  - The docs "recommend that you do not use this feature in production environments".
  
  — [DuckDB VSS docs](https://duckdb.org/docs/current/core_extensions/vss); [DuckDB VSS launch post (2024)](https://duckdb.org/2024/05/03/vector-similarity-search-vss)
- **Chroma.** It added first-class BM25 and SPLADE sparse vectors in November 2025, and its Search API supports weighted hybrid search. The official Rust crate is an HTTP client (`ChromaHttpClient::cloud()`). Local use goes through Python (`pip install chromadb` / `chroma run`). — [Chroma sparse vector support](https://www.trychroma.com/project/sparse-vector-search); [Chroma BM25 docs](https://docs.trychroma.com/integrations/embedding-models/chroma-bm25.md); [chroma crate docs.rs](https://docs.rs/crate/chroma/latest)
- **Qdrant Edge.** It is a "lightweight, in-process vector search engine" for devices, with optional sync to Qdrant Cloud.
  - Private beta was announced in July 2025. Trade press said it runs hybrid and multimodal search locally with no server process or background threads.
  - Mirrors list qdrant-edge-py 0.1.0 (Dec 2025) to 0.6.1 (Mar 2026) and the Rust crate at about 0.8.0 (around Aug/Sep 2026). These versions come from mirrors and were not checked against PyPI or crates.io.
  - I found no general-availability announcement.
  
  — [Qdrant Edge page](https://qdrant.tech/edge/); [Blocks & Files, Jul 2025](https://blocksandfiles.com/2025/07/29/qdrant-gets-the-vector-database-edge/); [CERN PyPI mirror](https://simple-repository.app.cern.ch/project/qdrant-edge-py); [ecosyste.ms summary](https://summary.ecosyste.ms/projects/378918)
- **pgvectorscale.** It adds a StreamingDiskANN index that keeps the graph on disk.
  - The vendor (Timescale/Tiger Data) claims, on 50M × 768-d Cohere embeddings at 99% recall, 28× lower p95 latency and 16× higher throughput than Pinecone s1 (vendor claim).
  - The latest release found was 0.9.0 (November 2025).
  - The licence is reported inconsistently: PostgreSQL License in one review, Apache-2.0 in another listing. Unverified.
  
  — [pgvectorscale GitHub](https://github.com/timescale/pgvectorscale); [Tiger Data blog](https://www.tigerdata.com/blog/pgvector-vs-pinecone); [aicoolies review](https://aicoolies.com/reviews/pgvectorscale-review); [SourcePulse](https://www.sourcepulse.org/projects/1446491)
- Hybrid BM25 inside Postgres without an Elasticsearch sidecar is discussed as feasible with pgvectorscale, but it requires a running server. — [thebuild.com](https://thebuild.com/blog/on-pgvectorscale-and-hybrid-search-without-an-elasticsearch-sidecar/)

### Inferences
- For 10^3–10^6 short items per user, every engine above trades rewire's zero-dependency `sqlite3` stdlib + numpy for a much larger dependency: Arrow/Lance, the DuckDB binary, a Chroma server or Python package, the Qdrant Edge crate, or a Postgres server.
- The main thing gained is ANN, which only matters above about 10^5–10^6 vectors (see §4).
- Any engine whose BM25 differs from FTS5 changes rankings. Examples are differences in tokenizer, IDF formula, or Tantivy's BM25 parameters. That would break the tuned SEM_WEIGHT / EP_SEM_WEIGHT values and the "identical in Python and Rust" property unless both implementations use the same engine library.
- Qdrant Edge, a Rust crate with Python bindings, is the most plausible "same engine in both languages" alternative. LanceDB also has Rust and Python SDKs. Neither was measured here.

### Gaps
- No independent, hardware-specified latency or footprint numbers for embedded LanceDB, Chroma or Qdrant Edge at 10^4–10^6. The docs pages could not be fetched (DNS blocked).
- Licences were not verified from primary sources: LanceDB (believed Apache-2.0), Chroma (believed Apache-2.0), Qdrant Edge, and pgvectorscale (conflicting).
- I don't know whether DuckDB has shipped WAL recovery for HNSW since 2024. The current docs still carry the warning.

## 3. Embedding choice vs database choice, and rerankers: which matters more?

### Takeaway
Retrieval quality is driven by the embedding/reranker and the fusion design, not the storage engine: the storage engines here all return the same exact top-k for brute force, and only ANN recall differs. Static models (Model2Vec/WordLlama-class) reach roughly 80% of a small transformer's MTEB retrieval score. Adding a cross-encoder reranker over hybrid candidates shows the largest measured gains.

### Cited Findings
- The best Model2Vec static retrieval model, potion-retrieval-32M, scores 35.06 on MTEB Retrieval, against 42.92 for all-MiniLM-L6-v2. The model card calls this about 81.7% of MiniLM. static-retrieval-mrl-en-v1 scores 34.95 and potion-base-32M 32.67. On the full MTEB English average, potion-base-32M scores 52.13 vs 55.93, and the project describes static inference as "up to 500× faster on CPU" (vendor). — [potion-retrieval-32M card](https://huggingface.co/minishlab/potion-retrieval-32M); [Model2Vec results](https://minish.ai/packages/model2vec/results/)
- bge-small-en-v1.5 scores a BEIR average nDCG@10 of 53.9, as reported in the Granite Embedding R2 paper. — [Granite Embedding R2](https://arxiv.org/pdf/2508.21085)
- text-embedding-3-large (3072-d, 8192 tokens) has an MTEB overall average of about 64.6, against 64.23 for bge-large-en-v1.5. That is an all-task average, not retrieval. Reported MTEB averages for bge-small conflict (62.17 vs 64.3), probably from different MTEB versions. — [Codesota](https://codesota.com/academy/lessons/1-1-text-embeddings); [mGTE paper](https://arxiv.org/pdf/2407.19669); [AWS listing](https://aws.amazon.com/marketplace/pp/prodview-ve2ckii4jsz3m); [GTM directory](https://preview.thegtmdirectory.com/models/baai-bge-small-en/bge-small-en-v1.5)
- On a hybrid BM25 + vector (RRF) pipeline (APS-Bench, 2026), a 0.6B reranker (jina-reranker-v3) lifted MRR@10 from 64.2 to 86.1, and a 4B reranker (qwen3-reranker) scored 85.7. A bigger reranker did not help. — [arXiv 2607.24663](https://arxiv.org/pdf/2607.24663)
- On MS MARCO, BM25 scores 0.1874 MRR@10, monoT5-3B 0.3980 and RankT5-Large 0.4222. Architecture and training mattered as much as size. Injecting BM25 scores into cross-encoder input beat plain interpolation. — [HYRR, arXiv 2212.10528](https://arxiv.org/pdf/2212.10528); [Springer IRJ 2024](https://link.springer.com/article/10.1007/s10791-024-09435-8)
- Waterloo (Lin et al.) argue that state-of-the-art results on popular benchmarks come only from combining good first-stage retrieval with reranking. — [Waterloo paper](https://cs.uwaterloo.ca/~jimmylin/publications/978-3-030-99736-6_44.pdf)
- Counter-evidence: a small blog experiment found reranking averaged −0.05% where hybrid already reached 98.3% top-10. Weak evidence. — [jaekang blog](https://jaekang.quarto.pub/jaekangai-quarto/posts/2411_rerank_rag/2024-11-23-rerank-rag.html)
- Vendor claim: ZeroEntropy reports +28% NDCG@10 from rerankers. Marketing-grade. — [ZeroEntropy](https://www.zeroentropy.dev/articles/ultimate-guide-to-choosing-the-best-reranking-model-in-2025/)
- rewire's own data, from the repo: the static 256-d vectors alone reach R@5 57% vs 90% for BM25 + keywords, and they help only as a small bonus. — `/home/user/rewire/src/rewire/index.py` (comments at SEM_WEIGHT, EP_SEM_WEIGHT)

### Inferences
- For exact (brute-force) search, all backends return the same neighbours for the same vectors, so changing the database cannot improve quality. Only the embedding, fusion, reranking and chunking can.
- Going from WordLlama 256-d to a contextual small encoder is the likely bigger quality lever. Candidates are bge-small (384-d), all-MiniLM (384-d) or potion-retrieval-32M (static, 256/512-d).
- A cross-encoder reranker over the top 20–50 hybrid candidates is the largest lever per the APS-Bench evidence. It costs ONNX/torch runtime, a model file and CPU latency per query, none of which were measured here.
- API embeddings (3072-d) bring network, cost, privacy and determinism problems for a local-first tool. Their MTEB average edge over bge-large is under one point.

### Gaps
- I found no controlled study comparing "upgrade the embedding" against "add a reranker" on conversational-memory benchmarks (LoCoMo, LongMemEval).
- No WordLlama MTEB retrieval figure was found in this session.
- The MTEB leaderboard was not consulted directly.

## 4. When does brute-force scan stop meeting <50 ms? (256-d vs 768-d vs 3072-d)

### Takeaway
With numpy BLAS on 4 Xeon vCPUs, the float32 matvec stays under 50 ms up to about 1M×256-d (46 ms), about 400k×768-d and about 100k×3072-d (40 ms). rewire's current full `argsort` adds about 100 ms at 1M, and `argpartition` would remove most of that. Through sqlite-vec's `vec0`, the 50 ms limit is hit already at about 100k×256-d.

### Cited Findings
- **Measured here: numpy float32 `m @ q`** (4 vCPU Xeon 2.8 GHz). The partition and sort columns are single noisy samples. The 104 ms at 10k is a first-call artefact.

  | dim | n | matrix size | matvec | argpartition top-20 | full argsort |
  |---|---|---|---|---|---|
  | 256 | 10k | 10 MB | 0.1 ms | (104 ms, first-call artefact) | 2.5 ms |
  | 256 | 100k | 102 MB | 4.1 ms | 5.6 ms | 7.1 ms |
  | 256 | 1M | 1.0 GB | 45.7 ms | 62 ms | 102 ms |
  | 768 | 10k | — | 0.7 ms | — | — |
  | 768 | 100k | 307 MB | 9.8 ms | — | — |
  | 768 | 1M | 3.1 GB | 123.5 ms | 29 ms | 70 ms |
  | 3072 | 10k | — | 2.8 ms | — | — |
  | 3072 | 100k | 1.2 GB | 40.0 ms | — | — |

  Source: own run.
- Measured here, sqlite-vec float32 k=20: 52 ms at 100k×256, 172 ms at 100k×768, 643 ms at 100k×3072 and 526 ms at 1M×256 (table in §1).
- Secondhand: sqlite-vec about 17 ms and NumPy about 136 ms at 1M×128-d on an M1. This contradicts the ranking measured here, where numpy was faster. Possible causes are a different numpy BLAS or a different sqlite-vec build. Unverified. — [Mixpeek guide](https://mixpeek.com/guides/do-you-need-a-vector-database)

### Inferences
- The scan is memory-bandwidth bound: time is roughly proportional to n × d × bytes. Halving the bytes with float16 storage, or quartering them with int8 plus rescoring, scales the thresholds accordingly (not measured).
- RAM is the real limit before latency. 1M × 3072-d float32 is 12 GB per user, against 1 GB at 256-d. A 3072-d API embedding therefore makes 10^6 items infeasible on a laptop without quantization or ANN.
- At rewire's realistic per-user scale, brute force at 256-d is far below 50 ms: under 1 ms to 4 ms of matvec. The scale is about 10^3–10^5 facts plus episode chunks; a LongMemEval_S history is about 50 sessions. ANN is unnecessary until roughly 10^6 vectors at 256-d or about 10^5 at 3072-d.
- One cheap win in the current code: `sims.argsort()[::-1]` sorts everything. `np.argpartition` to about 4×limit, followed by a sort, would remove the sort cost at large n. The scope filter must still be applied, for example by over-fetching.

### Gaps
- No laptop measurement was made: there was no Apple-silicon or consumer-x86 box. Laptop numbers should be re-run with the same script.
- Rust-side brute-force timing was not measured; ndarray/BLAS vs a plain loop are untested.

## 5. Trade-offs for local-first sync (git, multi-device), privacy, offline use, determinism

### Takeaway
rewire's design already has the right shape for local-first use: plain-text source of truth in git, plus a rebuildable stdlib-SQLite index that is never synced. Every alternative engine adds a binary format that should also stay derived and unsynced. Any ANN index, or any embedding computed by a non-deterministic or remote model, weakens the "identical results in two implementations" property.

### Cited Findings
- DuckDB VSS can corrupt the HNSW index after an unclean shutdown because WAL recovery is not implemented. That is acceptable only for an index that can be rebuilt. — [DuckDB VSS docs](https://duckdb.org/docs/current/core_extensions/vss)
- Qdrant Edge offers optional sync with Qdrant Cloud. That is a vendor-specific sync path, unlike git. — [Qdrant Edge](https://qdrant.tech/edge/)
- rewire: "Derived SQLite index: rebuildable at any time from nodes/ and events/". Ties are broken by id or path/turn so that rankings are "identical in every implementation". — `/home/user/rewire/src/rewire/index.py`
- rewire's semantic model needs "No server, no API, no download at run time". — `/home/user/rewire/src/rewire/semantic.py`

### Inferences
- **Determinism risks, in decreasing order:**
  1. ANN graphs (HNSW/DiskANN). Results depend on insertion order and parameters, and recall is under 100%.
  2. API embeddings. The model version can change server-side, outputs are not bit-reproducible, they need the network and they leak content.
  3. Floating-point summation order between numpy (BLAS, SIMD, multithreaded) and a Rust loop. The last-bit differences can swap near-tied ranks.
  
  Mitigations for the third risk:
  - Round similarities to a fixed precision, such as 1e-6, before ranking, and keep the id tiebreak.
  - Or store int8 vectors and compute integer dot products, which are exact and order-independent.
  - Or have both languages call one C kernel, such as sqlite-vec's `vec_distance_cosine`.
- **Multi-device sync.** Syncing the markdown via git and rebuilding the index on each device avoids binary merge conflicts. Embeddings must also be reproducible on each device, which a static model shipped in the wheel guarantees. A storage engine with a richer format adds no sync benefit.
- **Privacy and offline use.** Local static or small ONNX encoders and local rerankers keep everything offline. API embeddings (e.g. OpenAI 3072-d) break offline use and send memory content to a third party.
- **Bottom line for rewire.** At 10^3–10^6 items per user, keep SQLite FTS5 + brute-force vectors. Spend the effort on these instead:
  - a stronger local embedding (contextual small encoder or a better static model);
  - an optional local cross-encoder over hybrid candidates;
  - `argpartition`;
  - optional float16/int8 storage.
  
  Revisit ANN, via sqlite-vec if it gains an index or via Qdrant Edge or LanceDB, only if one user approaches 10^6 vectors or uses 768–3072-d vectors.

### Gaps
- No measured comparison of Python vs Rust score drift for float32 cosine on rewire's actual data.
- No sources were found on git-sync patterns specific to LanceDB, Qdrant Edge or Chroma data directories.
