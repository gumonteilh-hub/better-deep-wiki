import { createFileRoute } from "@tanstack/react-router";
import { memo, useEffect, useRef, useState } from "react";
import { fetchStreamedCompletion } from "../../service";
import { InstructionInput } from "../../components/InstructionInput";
import "../../Markdown.css"
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSanitize from "rehype-sanitize";
import rehypeRaw from "rehype-raw";
//@ts-ignore
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
//@ts-ignore
import { oneDark, prism } from 'react-syntax-highlighter/dist/esm/styles/prism';



export const Route = createFileRoute("/ask/$repo")({
  component: AskRepoComponent,
});

interface IConversation {
  chats: (IQuestion | IResponse)[]
}

interface IChat {
  content: string;
}

type IQuestion = IChat & {
  type: 'Q'
}

type IResponse = IChat & {
  type: 'R'
}


function AskRepoComponent() {
  const [prompt, setPrompt] = useState("");
  const { repo } = Route.useParams();
  const [loading, setLoading] = useState(false);
  const [instructions, setInstructions] = useState("");
  const [conversation, setConversation] = useState<IConversation>(() => loadFromStorage(repo))
  const [response, setResponse] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Scroll en bas à chaque ajout de message
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [response, loading]);

  useEffect(() => {
    setConversation(loadFromStorage(repo));
  }, [repo]);

  useEffect(() => {
    saveInStorage(repo, conversation)
  }, [repo, conversation])

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const trimmed = prompt.trim();
    if (trimmed) {
      setConversation(p => ({ chats: [...p.chats, { type: 'Q', content: prompt }] }));
      setLoading(true)
      setResponse("")
      let totalResponse = "";
      try {
        await fetchStreamedCompletion(
          repo,
          prompt,
          instructions,
          (chunk) => {
            setLoading(false);
            totalResponse = totalResponse + chunk
            setResponse((prev) => prev + chunk);
          }
        );
      } finally {
        setLoading(false);
        setConversation(p => ({ chats: [...p.chats, { type: 'R', content: totalResponse }] }));
        setResponse("");
      }
      setPrompt("");
    }
  };

  return (
    <div className="ask-repo">
      <header>
        <h1>{repo}</h1>
      </header>

      <main className='chat-container'>
        <MemoizedChat conversation={conversation} />
        {loading && <div className="loader-dots">
          <span className="loader-dot"></span>
          <span className="loader-dot"></span>
          <span className="loader-dot"></span>
        </div>}
        {!loading && response.length > 0 && <div className="chat response"><MemoizedMarkdownViewer markdown={response} ></MemoizedMarkdownViewer></div>}
        {!loading && conversation.chats.length == 0 && <p className="placeholder">Posez une question sur le dépôt…</p>}
        <div ref={bottomRef} />
      </main>

      <InstructionInput value={instructions} setValue={setInstructions} />
      <form onSubmit={handleSubmit}>
        <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} placeholder="Entrez votre question…" />
        <button type="submit" disabled={loading || !prompt.trim()}>
          Envoyer
        </button>
      </form>
    </div >
  );
}


const MemoizedChat = memo(function displayChat({ conversation }: { conversation: IConversation }) {
  return conversation.chats.map((chat, index) => {
    if (chat.type === 'Q') {
      return <div key={index} className="chat question">{chat.content}</div>
    } else {
      return <div key={index} className="chat response"><MemoizedMarkdownViewer markdown={chat.content}></MemoizedMarkdownViewer></div>
    }
  })
})

const MemoizedMarkdownViewer = memo(function MarkdownViewer({ markdown }: { markdown: string }) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[rehypeRaw, rehypeSanitize]}
      components={{
        code({ node, className, children, ...props }) {
          const match = /language-(\w+)/.exec(className ?? '');
          return match ? (
            <SyntaxHighlighter
              style={oneDark}
              PreTag="div"
              language={match[1]}
              {...props}
            >
              {String(children).replace(/\n$/, '')}
            </SyntaxHighlighter>
          ) : (
            <code className={className} {...props}>{children}</code>
          );
        },
      }}
    >
      {markdown}
    </ReactMarkdown>
  );
})

function loadFromStorage(repoName: string): IConversation {
  const conversationHistory = localStorage.getItem(repoName);

  if (!conversationHistory) return { chats: [] }
  try {
    const conversation: IConversation = JSON.parse(conversationHistory);
    return conversation;
  } catch {
    return { chats: [] }
  }

}

function saveInStorage(repoName: string, conversation: IConversation) {

  const truncatedStorage: IConversation = { chats: conversation.chats.slice(-10) }

  localStorage.setItem(repoName, JSON.stringify(truncatedStorage));
}
