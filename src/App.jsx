import { useState, useRef, useEffect } from "react";
import "./App.css";
import useChat from "./hooks/useChat.js";
import useToast from "./hooks/useToast.js";
import Toast from "./components/Toast.jsx";
import Sidebar from "./components/Sidebar.jsx";
import TopBar from "./components/TopBar.jsx";
import MessageList from "./components/MessageList.jsx";
import Composer from "./components/Composer.jsx";

function App() {
  const { toasts, addToast, removeToast } = useToast();
  const [input, setInput] = useState("");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const drafts = useRef({});
  const prevChatId = useRef(null);

  const {
    models,
    selectedModel,
    setSelectedModel,
    thinking,
    setThinking,
    chatList,
    activeChatId,
    activeMessages,
    isStreaming,
    loadingChat,
    endRef,
    inputRef,
    openChat,
    startNewChat,
    send,
  } = useChat(addToast);

  // Save draft on every input change (avoids stale closure)
  useEffect(() => {
    if (prevChatId.current !== null) {
      drafts.current[prevChatId.current] = input;
    }
  }, [input]);

  // Restore draft when switching chats
  useEffect(() => {
    if (activeChatId === prevChatId.current) return;
    prevChatId.current = activeChatId;
    const saved = drafts.current[activeChatId] || "";
    setInput(saved);
    // Reset textarea height after React re-renders with new value
    requestAnimationFrame(() => {
      if (inputRef.current) {
        if (saved) {
          inputRef.current.style.height = Math.min(inputRef.current.scrollHeight, 180) + "px";
        } else {
          inputRef.current.style.height = "21px";
        }
      }
    });
  }, [activeChatId]);

  const handleSend = () => {
    send(input, setInput);
    drafts.current[activeChatId] = "";
    if (inputRef.current) {
      inputRef.current.style.height = "21px";
    }
  };

  return (
    <div className="app">
      <div className="toast-container">
        {toasts.map((t) => (
          <Toast key={t.id} message={t.message} type={t.type} onClose={() => removeToast(t.id)} />
        ))}
      </div>

      <Sidebar
        chatList={chatList}
        activeChatId={activeChatId}
        onOpenChat={openChat}
        onNewChat={startNewChat}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
      />

      <main className="main">
        <TopBar
          selectedModel={selectedModel}
          models={models}
          onSelectModel={setSelectedModel}
          onToggleSidebar={() => setSidebarOpen(true)}
        />

        <MessageList
          messages={activeMessages}
          isStreaming={isStreaming}
          loadingChat={loadingChat}
          endRef={endRef}
        />

        <Composer
          input={input}
          setInput={setInput}
          isStreaming={isStreaming}
          selectedModel={selectedModel}
          onSend={handleSend}
          inputRef={inputRef}
          thinking={thinking}
          onToggleThinking={() => setThinking((p) => !p)}
        />
      </main>
    </div>
  );
}

export default App;
