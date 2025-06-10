import { createRootRoute, Outlet } from "@tanstack/react-router";
import Sidebar from "../components/Sidebar";
import { useEffect, useState } from "react";
import { listRepos } from "../service";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  const [repos, setRepos] = useState<string[]>([]);

  useEffect(() => {
    listRepos().then((res) => {
      setRepos(res.repos);
    });
  }, []);

  return (
    <div className="app-grid">
      <header className="app-header">
        <h1>Better deepwiki</h1>
      </header>

      <aside className="sidebar">
        <Sidebar repos={repos} />
      </aside>

      <main className="app-main">
        <Outlet />
      </main>
    </div>
  );
}
