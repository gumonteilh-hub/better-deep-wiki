import { createFileRoute } from "@tanstack/react-router";
// import { Sidebar } from "../components/Sidebar";
// import { ChatInput } from "../components/ChatInput";

export const Route = createFileRoute("/")({
  component: Home,
});

export default function Home() {
  return (
    <footer className="flex items-center justify-center py-3 px-4 bg-white border-t">
      <input type="text" />
    </footer>
  );
}
