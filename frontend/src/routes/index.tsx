import { createFileRoute, Link } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: Home,
});

export default function Home() {
  return (
    <h1>
      <Link to="/embedding" >Indexer un premier dépo</Link>
    </h1>
  );
}
