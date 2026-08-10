import { useState, useRef, useEffect, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

const OPENROUTER_ID_MAP = {
  "InclusionAi": "inclusionai/ling-3.0-tiny:free",
  "Poolside: Laguna S 2.1": "poolside/laguna-s-2.1:free",
  "Poolside: Laguna XS 2.1": "poolside/laguna-xs-2.1:free",
  "Cohere North mini": "cohere/north-mini-code:free",
  "Nvidia Nemotron 3.5 (content safety)": "nvidia/nemotron-3.5-content-safety:free",
  "Nvidia Nemotron 3 Ultra": "nvidia/nemotron-3-ultra-550b-a55b:free",
  "Nvidia Nemotron 3 Nano": "nvidia/nemotron-3-nano-30b-a3b:free",
  "Nvidia Nemotron Nano (9B)": "nvidia/nemotron-nano-9b-v2:free",
  "Nvidia Nemotron 3 Nano Omni": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
  "Nvidia Nemotron 3 Super": "nvidia/nemotron-3-super-120b-a12b:free",
  "Google Gemma 4": "google/gemma-4-31b-it:free",
  "Google Gemma 4 (A4B)": "google/gemma-4-26b-a4b-it:free",
  "OpenAI GPT OSS": "openai/gpt-oss-20b:free",
};

const THINKING_SUPPORTED = new Set([
  "Nvidia Nemotron 3 Nano Omni",
  "Google Gemma 4",
  "Google Gemma 4 (A4B)",
]);

const MAX_FILE_SIZE = 50 * 1024 * 1024;
const MAX_FILES = 10;

function load(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v ? JSON.parse(v) : fallback;
  } catch {
    return fallback;
  }
}

function save(key, val) {
  try {
    localStorage.setItem(key, JSON.stringify(val));
  } catch {}
}

function dateGroup(iso) {
  const d = new Date(iso);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dd = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  if (dd.getTime() === today.getTime()) return "Today";
  const yesterday = new Date(today - 86400000);
  if (dd.getTime() === yesterday.getTime()) return "Yesterday";
  if (dd > new Date(today - 7 * 86400000)) return "Previous 7 Days";
  return "Older";
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getFileIcon(name) {
  const ext = name.split(".").pop().toLowerCase();
  const map = {
    js: "JS", jsx: "JSX", ts: "TS", tsx: "TSX", py: "PY", rs: "RS",
    rb: "RB", go: "GO", java: "JV", c: "C", cpp: "C++", h: "H",
    css: "CSS", scss: "SC", html: "HTM", json: "JSON", xml: "XML",
    yaml: "YML", yml: "YML", md: "MD", txt: "TXT", sql: "SQL",
    sh: "SH", bash: "SH", ps1: "PS", dart: "DT", swift: "SW",
    kt: "KT", kts: "KT", vue: "VU", svelte: "SV", php: "PHP",
    png: "IMG", jpg: "IMG", jpeg: "IMG", gif: "IMG", webp: "IMG", svg: "IMG", bmp: "IMG", ico: "IMG",
    mp4: "VID", mov: "VID", avi: "VID", mkv: "VID", webm: "VID",
    mp3: "AUD", wav: "AUD", flac: "AUD", ogg: "AUD", aac: "AUD",
    pdf: "PDF", doc: "DOC", docx: "DOC", xls: "XLS", xlsx: "XLS",
    ppt: "PPT", pptx: "PPT", zip: "ZIP", rar: "ZIP", "7z": "ZIP", tar: "ZIP", gz: "ZIP",
    exe: "EXE", dmg: "DMG", iso: "ISO",
  };
  return map[ext] || "F";
}

function getFileColor(name) {
  const ext = name.split(".").pop().toLowerCase();
  const codeExts = new Set(["js","jsx","ts","tsx","py","rs","rb","go","java","c","cpp","h","css","scss","html","json","xml","yaml","yml","sh","bash","dart","swift","kt","kts","vue","svelte","php","sql"]);
  const imgExts = new Set(["png","jpg","jpeg","gif","webp","svg","bmp","ico"]);
  const vidExts = new Set(["mp4","mov","avi","mkv","webm"]);
  const audExts = new Set(["mp3","wav","flac","ogg","aac"]);
  const docExts = new Set(["pdf","doc","docx","txt","md"]);
  const sheetExts = new Set(["xls","xlsx","csv"]);
  const archiveExts = new Set(["zip","rar","7z","tar","gz"]);

  if (codeExts.has(ext)) return "#f59e0b";
  if (imgExts.has(ext)) return "#10b981";
  if (vidExts.has(ext)) return "#8b5cf6";
  if (audExts.has(ext)) return "#ec4899";
  if (docExts.has(ext)) return "#3b82f6";
  if (sheetExts.has(ext)) return "#22c55e";
  if (archiveExts.has(ext)) return "#f97316";
  return "#6b7280";
}

function validateFile(file) {
  if (file.size > MAX_FILE_SIZE) {
    return `${file.name} exceeds the ${formatBytes(MAX_FILE_SIZE)} size limit (${formatBytes(file.size)})`;
  }
  return null;
}

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
              <path d="M5 13l4 4L19 7" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          ) : (
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
              <rect x="9" y="9" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.8"/>
              <path d="M5 15V5a2 2 0 012-2h10" stroke="currentColor" strokeWidth="1.8"/>
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

function Toast({ message, type, onClose }) {
  useEffect(() => {
    const t = setTimeout(onClose, 4000);
    return () => clearTimeout(t);
  }, [onClose]);

  return (
    <div className={`toast toast-${type}`} onClick={onClose}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
        {type === "error" ? (
          <>
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.8"/>
            <path d="M15 9l-6 6M9 9l6 6" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </>
        ) : (
          <>
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.8"/>
            <path d="M9 12l2 2 4-4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </>
        )}
      </svg>
      <span>{message}</span>
    </div>
  );
}

function App() {
  const [models, setModels] = useState([]);
  const [selectedModel, setSelectedModel] = useState(null);
  const [thinking, setThinking] = useState(() => load("atlas_think", false));
  const [conversations, setConversations] = useState(() => load("atlas_convos", []));
  const [activeId, setActiveId] = useState(null);
  const [input, setInput] = useState("");
  const [files, setFiles] = useState([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [toasts, setToasts] = useState([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const [fileErrors, setFileErrors] = useState([]);

  const endRef = useRef(null);
  const inputRef = useRef(null);
  const fileRef = useRef(null);
  const dropRef = useRef(null);
  const abortedRef = useRef(false);
  const dragCounter = useRef(0);

  const activeConvo = conversations.find((c) => c.id === activeId);
  const msgs = activeConvo ? activeConvo.messages : [];

  const scroll = useCallback(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => { scroll(); }, [msgs, isStreaming, scroll]);
  useEffect(() => { inputRef.current?.focus(); }, [activeId]);
  useEffect(() => { save("atlas_convos", conversations); }, [conversations]);
  useEffect(() => { save("atlas_think", thinking); }, [thinking]);

  useEffect(() => {
    function outside(e) {
      if (dropdownOpen && dropRef.current && !dropRef.current.contains(e.target)) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", outside);
    return () => document.removeEventListener("mousedown", outside);
  }, [dropdownOpen]);

  const addToast = useCallback((message, type = "error") => {
    const id = Date.now() + Math.random();
    setToasts((p) => [...p, { id, message, type }]);
  }, []);

  const removeToast = useCallback((id) => {
    setToasts((p) => p.filter((t) => t.id !== id));
  }, []);

  const addFiles = useCallback((newFiles) => {
    const errors = [];
    const valid = [];
    const currentCount = files.length;

    for (const f of newFiles) {
      if (currentCount + valid.length >= MAX_FILES) {
        errors.push(`Maximum ${MAX_FILES} files allowed at a time`);
        break;
      }
      const err = validateFile(f);
      if (err) {
        errors.push(err);
      } else {
        valid.push(f);
      }
    }

    if (errors.length > 0) {
      setFileErrors(errors);
      errors.forEach((e) => addToast(e, "error"));
    }

    if (valid.length > 0) {
      setFiles((prev) => {
        const existing = new Set(prev.map((f) => f.name + f.size));
        const unique = valid.filter((f) => {
          const key = f.name + f.size;
          if (existing.has(key)) {
            addToast(`${f.name} is already attached`, "error");
            return false;
          }
          existing.add(key);
          return true;
        });
        return [...prev, ...unique];
      });
    }
  }, [files.length, addToast]);

  const removeFile = useCallback((index) => {
    setFiles((p) => p.filter((_, i) => i !== index));
    setFileErrors([]);
  }, []);

  useEffect(() => {
    if (fileErrors.length > 0) {
      const t = setTimeout(() => setFileErrors([]), 5000);
      return () => clearTimeout(t);
    }
  }, [fileErrors]);

  const handleDragEnter = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragOver(true);
    }
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragOver(false);
    }
  }, []);

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
    dragCounter.current = 0;

    if (isStreaming) {
      addToast("Cannot attach files while streaming", "error");
      return;
    }

    const droppedFiles = Array.from(e.dataTransfer.files);
    if (droppedFiles.length === 0) {
      addToast("No files were dropped", "error");
      return;
    }
    addFiles(droppedFiles);
  }, [isStreaming, addFiles, addToast]);

  useEffect(() => {
    const app = document.querySelector(".app");
    if (!app) return;
    app.addEventListener("dragenter", handleDragEnter);
    app.addEventListener("dragleave", handleDragLeave);
    app.addEventListener("dragover", handleDragOver);
    app.addEventListener("drop", handleDrop);
    return () => {
      app.removeEventListener("dragenter", handleDragEnter);
      app.removeEventListener("dragleave", handleDragLeave);
      app.removeEventListener("dragover", handleDragOver);
      app.removeEventListener("drop", handleDrop);
    };
  }, [handleDragEnter, handleDragLeave, handleDragOver, handleDrop]);

  const loadModels = async () => {
    let parsed = [];
    try {
      const raw = await invoke("list_models");
      parsed = (raw || []).map(([name, provider]) => ({
        name,
        provider: typeof provider === "string" ? provider : "Ollama",
        cloud: typeof provider === "string" ? provider === "OpenRouter" : false,
      }));
    } catch {
      parsed = Object.keys(OPENROUTER_ID_MAP).map((name) => ({
        name,
        provider: "OpenRouter",
        cloud: true,
      }));
    }
    setModels(parsed);
    const saved = load("atlas_model", null);
    if (saved) {
      const m = parsed.find((x) => x.name === saved.name && x.provider === saved.provider);
      if (m) { setSelectedModel(m); return; }
    }
    if (parsed.length > 0) setSelectedModel(parsed[0]);
  };

  useEffect(() => { loadModels(); }, []);

  useEffect(() => {
    if (selectedModel) save("atlas_model", { name: selectedModel.name, provider: selectedModel.provider });
  }, [selectedModel]);

  const getModelId = (model) => {
    if (model.cloud) return OPENROUTER_ID_MAP[model.name] || model.name;
    return model.name;
  };

  const newChat = () => {
    const id = "c" + Date.now();
    setConversations((p) => [{ id, title: "New chat", createdAt: new Date().toISOString(), messages: [] }, ...p]);
    setActiveId(id);
    setFiles([]);
    setFileErrors([]);
    setInput("");
    setSidebarOpen(false);
  };

  const openChat = (id) => {
    setActiveId(id);
    setFiles([]);
    setFileErrors([]);
    setInput("");
    setSidebarOpen(false);
  };

  const deleteChat = (id, e) => {
    e.stopPropagation();
    setActiveId((current) => (current === id ? null : current));
    setConversations((p) => p.filter((c) => c.id !== id));
  };

  const send = async () => {
    const text = input.trim();
    if ((!text && files.length === 0) || isStreaming || !selectedModel) return;

    let convoId = activeId;
    if (!convoId) {
      convoId = "c" + Date.now();
      setConversations((p) => [{
        id: convoId,
        title: text.length > 50 ? text.slice(0, 50) + "\u2026" : text || "File upload",
        createdAt: new Date().toISOString(),
        messages: [],
      }, ...p]);
      setActiveId(convoId);
    }

    const userMsg = { id: Date.now(), role: "user", text, files: files.map((f) => ({ name: f.name, size: f.size, type: f.type })) };
    const botId = Date.now() + 1;
    const botMsg = { id: botId, role: "bot", text: "", model: selectedModel.name };

    setConversations((prev) =>
      prev.map((c) => {
        if (c.id !== convoId) return c;
        const updated = { ...c, messages: [...c.messages, userMsg, botMsg] };
        if (c.messages.length === 0) {
          updated.title = text.length > 50 ? text.slice(0, 50) + "\u2026" : text || "File upload";
        }
        return updated;
      })
    );

    setInput("");
    setFiles([]);
    setFileErrors([]);
    setIsStreaming(true);
    abortedRef.current = false;

    try {
      const channel = new Channel();
      channel.onmessage = (token) => {
        if (abortedRef.current) return;
        setConversations((prev) =>
          prev.map((c) => {
            if (c.id !== convoId) return c;
            return {
              ...c,
              messages: c.messages.map((m) =>
                m.id === botId ? { ...m, text: m.text + token } : m
              ),
            };
          })
        );
      };

      await invoke("run_llm", {
        message: text,
        onEvent: channel,
        think: thinking && THINKING_SUPPORTED.has(selectedModel?.name),
        modelDetails: [getModelId(selectedModel), selectedModel.provider],
      });
    } catch (err) {
      if (abortedRef.current) return;
      let errMsg = typeof err === "string" ? err : err?.message || "Unknown error";
      if (errMsg.includes("transformCallback")) {
        errMsg = 'Not running in Tauri. Use "npm run tauri dev" instead of "npm run dev".';
      }
      setConversations((prev) =>
        prev.map((c) => {
          if (c.id !== convoId) return c;
          return {
            ...c,
            messages: c.messages.map((m) =>
              m.id === botId
                ? { ...m, text: m.text ? `${m.text}\n\nError: ${errMsg}` : `Error: ${errMsg}` }
                : m
            ),
          };
        })
      );
    } finally {
      setIsStreaming(false);
      setConversations((prev) =>
        prev.map((c) => {
          if (c.id !== convoId) return c;
          return {
            ...c,
            messages: c.messages.map((m) =>
              m.id === botId && !m.text ? { ...m, text: "No response received." } : m
            ),
          };
        })
      );
    }
  };

  const handleKey = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  };

  const stop = () => {
    abortedRef.current = true;
    setIsStreaming(false);
  };

  const handleFileInput = (e) => {
    const selected = Array.from(e.target.files);
    if (selected.length > 0) {
      if (isStreaming) {
        addToast("Cannot attach files while streaming", "error");
      } else {
        addFiles(selected);
      }
    }
    e.target.value = "";
  };

  const filtered = conversations.filter((c) =>
    c.title.toLowerCase().includes(search.toLowerCase())
  );
  const grouped = {};
  filtered.forEach((c) => {
    const g = dateGroup(c.createdAt);
    (grouped[g] ||= []).push(c);
  });

  return (
    <div className="app">
      {sidebarOpen && (
        <div className="sidebar-overlay" onClick={() => setSidebarOpen(false)} />
      )}

      {isDragOver && (
        <div className="drop-overlay">
          <div className="drop-overlay-content">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M17 8l-5-5-5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M12 3v12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
            <span>Drop files to attach</span>
          </div>
        </div>
      )}

      <div className="toast-container">
        {toasts.map((t) => (
          <Toast key={t.id} message={t.message} type={t.type} onClose={() => removeToast(t.id)} />
        ))}
      </div>

      <aside className={`sidebar ${sidebarOpen ? "open" : ""}`}>
        <div className="sidebar-header">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round"/>
          </svg>
          Atlas
        </div>
        <button className="new-chat-btn" onClick={newChat}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M12 5v14M5 12h14" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          New chat
        </button>
        <div className="search-box">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
            <circle cx="11" cy="11" r="7" stroke="currentColor" strokeWidth="2"/>
            <path d="M21 21l-4.3-4.3" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          <input
            type="text"
            placeholder="Search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="history">
          {Object.entries(grouped).map(([label, items]) => (
            <div key={label}>
              <div className="history-group-label">{label}</div>
              {items.map((c) => (
                <div
                  key={c.id}
                  className={`history-item ${c.id === activeId ? "active" : ""}`}
                  onClick={() => openChat(c.id)}
                >
                  <svg className="history-item-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                  <span className="history-item-title">{c.title}</span>
                  <button className="history-item-delete" onClick={(e) => deleteChat(c.id, e)}>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                      <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                  </button>
                </div>
              ))}
            </div>
          ))}
          {conversations.length === 0 && (
            <div className="history-empty">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" style={{opacity: 0.3, marginBottom: "8px"}}>
                <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
              No conversations yet
            </div>
          )}
        </div>
      </aside>

      <main className="main">
        <div className="topbar">
          <button className="menu-toggle" onClick={() => setSidebarOpen(true)}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
              <path d="M4 6h16M4 12h16M4 18h16" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
            </svg>
          </button>

          <div ref={dropRef} className={`model-selector ${dropdownOpen ? "open" : ""}`}>
            <button className="model-trigger" onClick={() => setDropdownOpen(!dropdownOpen)}>
              {selectedModel?.cloud ? (
                <svg className="model-type-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M6.5 19a4.5 4.5 0 01-.4-8.98A5.5 5.5 0 0116.5 8.5a4 4 0 01-.5 7.97" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              ) : (
                <svg className="model-type-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <rect x="2" y="3" width="20" height="14" rx="2" stroke="currentColor" strokeWidth="2"/>
                  <path d="M8 21h8M12 17v4" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                </svg>
              )}
              {selectedModel?.cloud && <span className="model-dot online" />}
              <span className="model-name">{selectedModel ? selectedModel.name : "Select model"}</span>
              <svg className="model-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none">
                <path d="M6 9l6 6 6-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>

            <div className="model-dropdown">
              {models.length === 0 && (
                <div className="model-dropdown-empty">No models found</div>
              )}
              {models.map((m) => (
                <div
                  key={m.name + m.provider}
                  className={`model-option ${selectedModel?.name === m.name && selectedModel?.provider === m.provider ? "selected" : ""}`}
                  onClick={() => { setSelectedModel(m); setDropdownOpen(false); }}
                >
                  <div className="model-option-info">
                    <div className="model-option-name">
                      <span className="model-option-name-text">{m.name}</span>
                      {m.cloud ? (
                        <svg className="model-type-icon-sm" width="12" height="12" viewBox="0 0 24 24" fill="none">
                          <path d="M6.5 19a4.5 4.5 0 01-.4-8.98A5.5 5.5 0 0116.5 8.5a4 4 0 01-.5 7.97" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                        </svg>
                      ) : (
                        <svg className="model-type-icon-sm" width="12" height="12" viewBox="0 0 24 24" fill="none">
                          <rect x="2" y="3" width="20" height="14" rx="2" stroke="currentColor" strokeWidth="2"/>
                          <path d="M8 21h8M12 17v4" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                        </svg>
                      )}
                      {m.cloud && <span className="model-dot online" />}
                    </div>
                    <div className="model-option-provider">{m.provider}</div>
                  </div>
                  <svg className="model-option-check" width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M5 13l4 4L19 7" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </div>
              ))}
            </div>
          </div>

          <div className="topbar-spacer" />
        </div>

        <div className="messages">
          <div className="messages-inner">
            {msgs.length === 0 ? (
              <div className="empty-state">
                <div className="empty-state-icon">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none">
                    <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round"/>
                  </svg>
                </div>
                <h2>What can I help with?</h2>
                <p>Start a conversation or drag & drop files below.</p>
              </div>
            ) : (
              msgs.map((msg) => (
                <div key={msg.id} className={`message ${msg.role}`}>
                  {msg.role === "bot" && (
                    <div className="msg-avatar">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                        <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round"/>
                      </svg>
                    </div>
                  )}
                  <div className="msg-content">
                    {msg.role === "bot" && <div className="msg-label">{msg.model || "Atlas"}</div>}
                    {msg.files?.length > 0 && (
                      <div className="file-chips">
                        {msg.files.map((f, i) => {
                          const fileObj = typeof f === "string" ? { name: f } : f;
                          return (
                            <div className="file-chip" key={i}>
                              <span className="file-chip-icon" style={{ background: getFileColor(fileObj.name) }}>
                                {getFileIcon(fileObj.name)}
                              </span>
                              <span className="file-chip-name">{fileObj.name}</span>
                              {fileObj.size && <span className="file-chip-size">{formatBytes(fileObj.size)}</span>}
                            </div>
                          );
                        })}
                      </div>
                    )}
                    {msg.role === "user" ? (
                      <div className="msg-text user-bubble">{msg.text || <em style={{opacity: 0.7}}>Sent files</em>}</div>
                    ) : (
                      <>
                        {msg.text === "" ? (
                          <div className="typing-dots"><span /><span /><span /></div>
                        ) : (
                          <div className="msg-text"><BotMessage text={msg.text} /></div>
                        )}
                        {msg.text && !msg.text.startsWith("Error:") && (
                          <div className="msg-actions">
                            <button className="msg-action" onClick={() => navigator.clipboard?.writeText(msg.text)} title="Copy">
                              <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
                                <rect x="9" y="9" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.8"/>
                                <path d="M5 15V5a2 2 0 012-2h10" stroke="currentColor" strokeWidth="1.8"/>
                              </svg>
                            </button>
                          </div>
                        )}
                      </>
                    )}
                  </div>
                </div>
              ))
            )}
            <div ref={endRef} />
          </div>
        </div>

        <div className="composer">
          <div className="composer-inner">
            {files.length > 0 && (
              <div className="file-preview">
                {files.map((f, i) => (
                  <div className="file-preview-item" key={i}>
                    <span className="file-type-badge" style={{ background: getFileColor(f.name) }}>
                      {getFileIcon(f.name)}
                    </span>
                    <div className="file-preview-info">
                      <span className="file-preview-name">{f.name}</span>
                      <span className="file-preview-size">{formatBytes(f.size)}</span>
                    </div>
                    <button className="file-preview-remove" onClick={() => removeFile(i)}>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
                        <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"/>
                      </svg>
                    </button>
                  </div>
                ))}
              </div>
            )}

            {fileErrors.length > 0 && (
              <div className="file-errors">
                {fileErrors.map((err, i) => (
                  <div className="file-error-item" key={i}>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.8"/>
                      <path d="M12 8v4M12 16h.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                    <span>{err}</span>
                  </div>
                ))}
              </div>
            )}

            <div className="input-row">
              <button className="icon-btn" onClick={() => fileRef.current?.click()} title="Attach file">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path d="M21.44 11.05l-9.19 9.19a5 5 0 01-7.07-7.07l9.19-9.19a3.5 3.5 0 014.95 4.95l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </button>
              <input type="file" ref={fileRef} multiple hidden onChange={handleFileInput} />
              <textarea
                ref={inputRef}
                placeholder="Message Atlas"
                value={input}
                onChange={(e) => {
                  setInput(e.target.value);
                  const ta = e.target;
                  ta.style.height = "auto";
                  ta.style.height = Math.min(ta.scrollHeight, 180) + "px";
                }}
                onKeyDown={handleKey}
                rows={1}
              />
              {isStreaming ? (
                <button className="stop-btn" onClick={stop} title="Stop">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <rect x="6" y="6" width="12" height="12" rx="2"/>
                  </svg>
                </button>
              ) : (
                <button className="send-btn" onClick={send} disabled={(!input.trim() && files.length === 0) || !selectedModel} title="Send">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M12 19V5M12 5l-6 6M12 5l6 6" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </button>
              )}
            </div>

            <div className="input-meta">
              <div className="thinking-toggle" onClick={() => setThinking(!thinking)}>
                <div className={`toggle-switch ${thinking ? "on" : ""}`}>
                  <div className="toggle-knob" />
                </div>
                <span className={`thinking-label ${thinking ? "on" : ""}`}>Thinking</span>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
