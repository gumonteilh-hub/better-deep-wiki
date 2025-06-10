import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { getIndexableRepos, scanRepo } from "../service";

export const Route = createFileRoute("/embedding")({
  component: Embedding,
});

function Embedding() {
  const [availableRepos, setAvailablesRepos] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [repoIdentifier, setRepoIdentifier] = useState("");
  const navigate = useNavigate()

  const fetchRepos = async () => {
    setLoading(true);
    setError(null);

    getIndexableRepos().then((res) => {
      setLoading(false);
      setAvailablesRepos(res.repos);
    });
  };

  useEffect(() => {
    fetchRepos();
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!repoIdentifier) return;
    scanRepo(repoIdentifier).then(res => {
      navigate({to: `/ask/${res.repo_identifier}`})
    })
  };

  return (
    <div className="embedding-page">
      <section className="intro">
        <h1>Préparation de l'indexation</h1>
        <p>
          Pour indexer votre code, clonez d'abord le dépôt dans <code className="path">/clone/mon_repo</code>.
        </p>
        <p>Sélectionnez ensuite un dossier candidat.</p>
      </section>

      <section className="repos-section">
        <div className="repos-header">
          <h2>Dépôts détectés</h2>
          <button onClick={fetchRepos} disabled={loading} aria-label="Rafraîchir la liste">
            {loading ? "Chargement…" : "Rafraîchir"}
          </button>
        </div>

        {error && <p className="error">{error}</p>}

        {!loading && availableRepos.length === 0 && <p className="empty">Aucun dépôt trouvé dans /clone.</p>}

        <ul className="repo-list">
          {availableRepos.map((repo) => (
            <li key={repo} className={`repo-item ${repoIdentifier === repo ? "selected" : ""}`} onClick={() => setRepoIdentifier(repo)}>
              {repo}
            </li>
          ))}
        </ul>
      </section>

      {repoIdentifier && (
        <form onSubmit={handleSubmit} className="repo-form">
          <button type="submit">Scanner : {repoIdentifier}</button>
        </form>
      )}
    </div>
  );
}
