import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

function CodeBlock({ language, code }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard?.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <div className="code-block">
      <div className="code-block-header">
        <span className="code-block-lang">{language || "code"}</span>
        <button className="code-block-copy" onClick={copy}>
          {copied ? (
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
              <path d="M5 13l4 4L19 7" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          ) : (
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
              <rect x="9" y="9" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.8" />
              <path d="M5 15V5a2 2 0 012-2h10" stroke="currentColor" strokeWidth="1.8" />
            </svg>
          )}
        </button>
      </div>
      <div className="code-block-body">
        <SyntaxHighlighter
          language={language || "text"}
          style={oneDark}
          PreTag="div"
          customStyle={{
            margin: 0,
            borderRadius: 0,
            background: "#141414",
            fontSize: "13px",
            padding: "12px 16px",
          }}
          codeTagProps={{
            style: {
              fontFamily: '"SF Mono", "Fira Code", "Cascadia Code", Consolas, monospace',
            },
          }}
        >
          {code}
        </SyntaxHighlighter>
      </div>
    </div>
  );
}

function MarkedText({ text }) {
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          pre({ children }) {
            return <>{children}</>;
          },
          code({ className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || "");
            const codeStr = String(children).replace(/\n$/, "");
            if (match || codeStr.includes("\n")) {
              return <CodeBlock language={match?.[1] || ""} code={codeStr} />;
            }
            return <code className="inline-code" {...props}>{children}</code>;
          },
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}

function BotMessage({ text }) {
  if (!text) return null;
  if (text.startsWith("Error:")) {
    return <div className="msg-error-block">{text}</div>;
  }
  const errorIdx = text.lastIndexOf("\n\nError:");
  if (errorIdx !== -1) {
    return (
      <>
        <MarkedText text={text.slice(0, errorIdx)} />
        <div className="msg-error-block">{text.slice(errorIdx)}</div>
      </>
    );
  }
  return <MarkedText text={text} />;
}

export { CodeBlock, MarkedText, BotMessage };
