import axios from "axios";

// Base axios instance
const api = axios.create({
  baseURL: "/api",
  timeout: 200000,
});

// Types
export interface ScanResponse {
  repo_identifier: string;
}

export interface AskResponse {
  answer: string;
}

export interface RepoListResponse {
  repos: string[];
}

// Service functions
/**
 * Index a repository on the server
 * @param repoPath - absolute or relative path to the repo
 */
export function scanRepo(repoPath: string): Promise<ScanResponse> {
  return api.post<ScanResponse>("/scan_repo", { repo_path: repoPath }).then((res) => res.data);
}

/**
 * Ask a question against an indexed repository
 * @param repo_identifier - identifier of the indexed repo
 * @param question - user question
 * @param instructions - system instructions for the LLM
 */
export function askRepo(repo_identifier: string, question: string, instructions: string): Promise<AskResponse> {
  return api.post<AskResponse>("/ask_repo", { question, instructions, repo_identifier }).then((res) => res.data);
}

export async function fetchStreamedCompletion(
  repo_identifier: string, question: string, instructions: string,
  onChunk: (text: string) => void
): Promise<void> {
  const res = await fetch("/api/ask_repo", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ question, instructions, repo_identifier }),
  });

  if (!res.body) throw new Error("No response body");

  const reader = res.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { value, done } = await reader.read();
    if (value) {
      const text = decoder.decode(value, { stream: true });
      onChunk(text);
    }
    if (done) break;
  }
}


/**
 * Retrieve the list of indexed repositories
 */
export function listRepos(): Promise<RepoListResponse> {
  return api.get<RepoListResponse>("/repos").then((res) => res.data);
}

/**
 * Retrieve the list of indexable repositories
 */
export function getIndexableRepos(): Promise<RepoListResponse> {
  return api.get<RepoListResponse>("/indexable-repos").then((res) => res.data);
}
