# Better DeepWiki Roadmap (Rust)

> **Philosophy:** opinionated simplicity, robustness, and performance; every milestone must deliver direct value without adding unnecessary complexity.

---

## 0️⃣ Completed – v0.1 ✅

- **CLI application:** minimal RAG (repository traversal, embeddings, question/answer)

---

## 2️⃣ Completed – v0.3 – Graphical Interface ✅

- HTTP API served by **axum**
- Minimal **React** UI written in **TypeScript**: query field, answer display, highlighted context

---

## 1️⃣ Completed – v0.2 – Dockerization ✅

- Official lightweight container bundling the Rust binary **and an embedded Qdrant instance** for vector storage
- One‑command deployment with `docker compose`

---

## 3️⃣ v0.4 – RAG Optimisation & OpenAI Support

- Option to use OpenAI embeddings and completions
- Improved ranking with tunable parameters
- **Explanatory schema generation at embedding time** to visualise key codebase features

---

## 4️⃣ v0.5 – Sessions & Context

- Session mechanism enabling **iterative queries**, carrying intermediate answers forward as context
- Configurable TTL and memory budget

---

## 5️⃣ v0.6 – Deep‑Searching

- Iterative queries ("rounds") to refine answers
- `--depth` parameter to control search depth
- Guardrails to avoid loops and excessive cost

---

## 🌐 Cross‑cutting tasks

| Axis | Description |
|------|-------------|
| Security | Dependency audit, secret management |
| DX | Readable logs, actionable errors, examples |
| Documentation | Guides, architecture diagrams, changelog |

---

*Roadmap is indicative and subject to change.*