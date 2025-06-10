import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { askRepo } from "../../service";
import { InstructionInput } from "../../components/InstructionInput";
import "../../Markdown.css"
import ReactMarkdown from "react-markdown";

export const Route = createFileRoute("/ask/$repo")({
  component: AskRepoComponent,
});

const markdownSample = `
Les équipes sont sauvegardées en appelant la fonction \`handleSaveTeam\` avec un objet contenant l'ID, le nom et les personnages de l'équipe. Cette fonction est appelée dans le composant \`TeamSelection\` lorsque l'utilisateur clique sur le bouton "Save".

Voici les sources qui permettent de répondre :

- Dans \`TeamSelection.tsx\`, la fonction \`saveCurrentTeam\` est définie pour gérer la validation et l'appel de \`handleSaveTeam\` :
  \`\`\`typescript
  function saveCurrentTeam(): void {
      let errorMessages: string[] = [];
      if (teamName == "") {
          errorMessages = [...errorMessages, "you need to give a name to your team"]
      }
      if (team.leaders.length !== 2 || team.characters.length !== 5) {
          errorMessages = [...errorMessages, "you must select a team of 7 characters"]
      }
      if (errorMessages.length > 0) {
          errorMessages.forEach(msg => {
              console.log(msg)
              toast.error(msg, { role: 'error' });
          })
          return;
      }
      handleSaveTeam({
          id: uuidv4(),
          name: teamName,
          characters: [...team.leaders, ...team.characters],
      });
      setTeam({ leaders: [], characters: [] });
      setTeamName("");
  }
  \`\`\`

- Le bouton "Save" appelle \`saveCurrentTeam\` lorsqu'il est cliqué :
  \`\`\`typescript
  <button className="saveButton" onClick={saveCurrentTeam}>
      Save
  </button>
  \`\`\`

- La structure de l'équipe à sauvegarder est définie dans \`team.ts\` :
  \`\`\`typescript
  export interface ITeamWithName {
      id: string;
      name: string;
      characters: Card[]
  }
  \`\`\`

Ces informations montrent clairement comment les équipes sont sauvegardées dans l'application.
`;

function AskRepoComponent() {
  const [prompt, setPrompt] = useState("");
  const { repo } = Route.useParams();
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState("");
  const [instructions, setInstructions] = useState("");

  useEffect(() => {
    setResponse("");
  }, [repo]);

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const trimmed = prompt.trim();
    if (trimmed) {
      askRepo(repo, prompt, instructions)
        .then((res) => {
          console.log(res.answer);
          setResponse(res.answer);
          setLoading(false);
        })
        .catch((err) => {
          console.error(err);
        });
      setPrompt("");
    }
  };

  return (
    <div className="ask-repo">
      <header>
        <h1>{repo}</h1>
      </header>

      <main>{loading ? <p>Chargement…</p> : markdownSample ? <pre><ReactMarkdown>{markdownSample}</ReactMarkdown></pre> : <p className="placeholder">Posez une question sur le dépôt…</p>}</main>

      <InstructionInput value={instructions} setValue={setInstructions} />
      <form onSubmit={handleSubmit}>
        <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} placeholder="Entrez votre question…" />
        <button type="submit" disabled={loading || !prompt.trim()}>
          Envoyer
        </button>
      </form>
    </div>
  );
}


function basicMarkdownToHtml(md: string): string {
  // Les blocs de code (```code```)
  md = md.replace(/```([\s\S]*?)```/g, '<pre class="code-block"><code>$1</code></pre>');
  // Le gras
  md = md.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  // L’italique
  md = md.replace(/\*(.*?)\*/g, '<em>$1</em>');
  // Les titres (##)
  md = md.replace(/^## (.*?)$/gm, '<h2>$1</h2>');
  md = md.replace(/^# (.*?)$/gm, '<h1>$1</h1>');
  // Les liens
  md = md.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');
  // Les listes à puces
  md = md.replace(/^\s*-\s(.*)$/gm, '<li>$1</li>');
  md = md.replace(/(<li>.*<\/li>)/gs, '<ul>$1</ul>');
  // Les citations >
  md = md.replace(/^>\s?(.*)$/gm, '<blockquote>$1</blockquote>');
  // Les retours à la ligne
  md = md.replace(/\n/g, '<br />');
  return md;
}

export function MarkdownRenderer({ children }: { children: string }) {
  const html = basicMarkdownToHtml(children);

  return (
    <div
      className="markdown-body"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}