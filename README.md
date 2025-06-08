# Better DeepWiki — Quick‑Start Guide

> **Purpose** — Better DeepWiki is a minimalist, opinionated Rust rewrite of *deepwiki‑open*. It enriches source code with Retrieval‑Augmented Generation (RAG) so that professional developers can ask high‑fidelity questions about any Git repository.

---

## 1 · Prerequisites

| Tool | Recommended Version | Purpose |
|------|---------------------|---------|
| **Rust** | stable ≥ 1.77 | Compiles the `better-deep-wiki` binary |
| **Docker** | ≥ 24.x | Runs Qdrant (vector database) |
| **Qdrant** | image `qdrant/qdrant` | Stores embeddings |
| **MistralAI API key** | — | Generates embeddings & answers |

> Better DeepWiki only supports MistralAI APIs for now.

---

## 2 · Launch Qdrant Locally

```bash
# Create a persistent Docker volume
$ docker volume create qdrant_data

# Start Qdrant (HTTP 6333, gRPC 6334)
$ docker run \
    -p 6333:6333 -p 6334:6334 \
    -v qdrant_data:/qdrant/storage \
    qdrant/qdrant
```

Keep this container running for the remainder of the session.

---

## 3 · Build Better DeepWiki

```bash
# Inside the project root
$ cargo build --release

# Resulting binary
./target/release/better-deep-wiki
```

*Omit `--release` for faster, non‑optimised builds during development.*

---

## 4 · Environment Configuration

1. Duplicate the example file:
   ```bash
   $ cp .env.bak .env
   ```
2. Edit `.env` and set at least one API key:
   ```env
   MISTRAL_API_KEY="your‑mistral‑key"
   ```
3. (Optional) Adjust the embedding model.

---

## 5 · Prepare the Repository to Analyse

Clone (or copy) your target repository into the **`clone/`** directory at the project’s root:

```bash
$ git clone git@github.com:acme/repo_test.git clone/repo_test
```

> Better DeepWiki deliberately avoids performing the clone itself; it only accepts a local path.

---

## 6 · Generate Embeddings

```bash
./target/release/better-deep-wiki embed \
  --repo-path clone/repo_test
```

The command:
1. Recursively scans the repository.
2. Splits files into chunks.
3. Sends each chunk for embedding.
4. Persists vectors in Qdrant.

---

## 7 · Query the Indexed Repository

```bash
./target/release/better-deep-wiki query \
  --question "What does the scheduler module do?" \
  --instructions "Answer concisely in markdown" \
  --repo-path clone/repo_test
```

- **--question** — the information you seek about the codebase.  
- **--instructions** — optional guidance on tone, language, or format.

The RAG pipeline selects the most relevant chunks and feeds them, along with your question, to the LLM.

---

## 8 · Helpful Commands

| Description | Command |
|-------------|---------|
| Global help | `better-deep-wiki --help` |
| Embed module help | `better-deep-wiki embed --help` |
| Query module help | `better-deep-wiki query --help` |

---

## 9 · Troubleshooting

| Symptom | Likely Cause |
|---------|--------------|
| `connection refused: 6333` | Qdrant container is not running |
| API *rate limit* errors | Reduce batch sizes or increase back‑off timing |
| `repo path not found` | `--repo-path` does not point to an existing local directory |

---

## 10 · Philosophy & Limitations

Better DeepWiki favours the *Unix philosophy*: a single, explicit workflow with minimal hidden behaviour. If you need to index multiple versions of a project, run separate instances (e.g. `clone/repo_v1`, `clone/repo_v2`).

---

## 11 · Licence

MIT — see `LICENSE`.

---

*Happy hacking!* 🚀