# Better DeepWiki — Quick‑Start Guide

> **Purpose** — Better DeepWiki is a minimalist, opinionated Rust rewrite of *deepwiki‑open*. It enriches source code with Retrieval‑Augmented Generation (RAG) so that professional developers can ask high‑fidelity questions about any Git repository.

---

## 1 · Prerequisites

| Tool | Recommended Version | Purpose |
|------|---------------------|---------|
| **Docker** | ≥ 24.x | Runs the entire application stack |
| **Docker Compose** | ≥ 2.x | Orchestrates services |
| **MistralAI API key** | — | Generates embeddings & answers |

---

## 2 · Configuration

1. Duplicate the example env file:
   ```bash
   cp .env.bak .env
   ```
2. Edit `.env` and set your Mistral API key:
   ```env
   MISTRAL_API_KEY="your-mistral-key"
   ```

---

## 3 · Prepare the Repository to Analyse

Clone (or copy) your target repository into the `clone/` directory at the project's root:

```bash
git clone git@github.com:acme/repo_test.git clone/repo_test
```

---

## 4 · Start the Application

### **Production Mode**
```bash
docker-compose up
```

### **Development Mode** (with hot reload)
```bash
docker-compose -f docker-compose.dev.yml up
```
The application will be accessible at **http://localhost**

### **Architecture**
- **Nginx Proxy**: Single entry point routing requests
- **Frontend**: React application with Vite (hot reload in dev mode)
- **Backend**: Rust API (hot reload in dev mode)
- **Qdrant**: Vector database for embeddings
- **Routes**:
  - `/` → Frontend
  - `/api/*` → Backend API  
  - `/qdrant/*` → Qdrant dashboard/API

---

## 5 · How to Use

### **Step 1 — Index a Repository**

- On the web UI, select a repository present in `/clone/` and start the indexing process.
- The interface shows progress and confirms when embedding is done.

<p align="center">
  <img src="screenshots/indexation-exemple.png" width="700" alt="Indexation example screenshot">
</p>

---

### **Step 2 — Ask Questions**

- Once the repository is indexed, type your questions into the prompt area.
- Answers are generated using context from your codebase.

<p align="center">
  <img src="screenshots/question-exemple.png" width="700" alt="Question example screenshot">
</p>

---

## 7 · Troubleshooting

| Symptom | Likely Cause |
|---------|--------------|
| `connection refused: 6334` | Qdrant container is not running |
| API *rate limit* errors | Reduce usage or check your API quota |
| Repository not listed | Ensure your repo is present in the `/clone/` directory |

---

## 8 · Philosophy & Limitations

Better DeepWiki follows the *Unix philosophy*: a single, explicit workflow with minimal hidden behaviour. No internal repo cloning. One repo = one indexation.

---

## 9 · Licence

MIT — see `LICENSE`.

---

*Happy hacking!* 🚀