import { useState, useCallback, useEffect } from "react";
import "./App.css";
import useChat from "./hooks/useChat";
import useToast from "./hooks/useToast";
import Toast from "./components/Toast";
import Sidebar from "./components/Sidebar";
import TopBar from "./components/TopBar";
import MessageList from "./components/MessageList";
import Composer from "./components/Composer";

function App() {
  const { toasts, addToast, removeToast } = useToast();
  const [input, setInput] = useState("");
  const [sidebarOpen, setSidebarOpen] = useState(false);

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
    saveDraft,
    getDraft,
  } = useChat(addToast);

  // Reset textarea height helper
  const resetTextareaHeight = useCallback(() => {
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
    }
  }, [inputRef]);

  // When activeChatId changes, restore draft or clear input
  useEffect(() => {
    const draft = getDraft(activeChatId);
    setInput(draft);
    resetTextareaHeight();
  }, [activeChatId, getDraft, resetTextareaHeight]);

  // Save draft whenever input changes
  useEffect(() => {
    saveDraft(activeChatId, input);
  }, [input, activeChatId, saveDraft]);

  const handleSend = () => {
    send(input, setInput);
    resetTextareaHeight();
  };

  // Open chat — save draft first is handled by the useEffect above
  const handleOpenChat = useCallback((id) => {
    openChat(id);
  }, [openChat]);

  // New chat
  const handleNewChat = useCallback(() => {
    startNewChat();
  }, [startNewChat]);

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
        onOpenChat={handleOpenChat}
        onNewChat={handleNewChat}
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