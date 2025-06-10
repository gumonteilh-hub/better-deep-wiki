import { Link } from "@tanstack/react-router";

export interface SidebarProps {
  repos: string[];
  currentRepo?: string;
}

export default function Sidebar({ repos, currentRepo }: SidebarProps) {
  return (
    <nav className="sidebar">
      <div>
        <h2>Repos indexés</h2>
        {repos.length === 0 ? (
          <p>Aucun dépôt indexé</p>
        ) : (
          <ul>
            {repos.map((repo) => (
              <li key={repo}>
                <Link
                  to="/ask/$repo"
                  params={{ repo }}
                  className={
                    repo === currentRepo ? "active" : undefined
                  }
                >
                  <span>{repo}</span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </div>

      <Link to="/embedding" className="add-repo">
        + Ajouter un repo
      </Link>
    </nav>
  );
}
