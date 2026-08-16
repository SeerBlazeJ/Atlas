import { useState, useRef, useEffect, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { THINKING_MANDATORY, OPENROUTER_ID_MAP } from "../constants/models.js";

export default function useChat(addToast) {
  const [models, setModels] = useState([]);
  const [selectedModel, setSelectedModel] = useState(null);
  const [thinking, setThinking] = useState(false);
  const [chatList, setChatList] = useState([]);
  const [activeChatId, setActiveChatId] = useState(null);
  const [activeMessages, setActiveMessages] = useState([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [loadingChat, setLoadingChat] = useState(false);

  // Per-chat draft storage
  const [drafts, setDrafts] = useState({});

  const endRef = useRef(null);
  const inputRef = useRef(null);

  const scroll = useCallback(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scroll();
  }, [activeMessages, isStreaming, scroll]);

  useEffect(() => {
    inputRef.current?.focus();
  }, [activeChatId]);

  // Auto-enable thinking for mandatory models only
  useEffect(() => {
    if (selectedModel && THINKING_MANDATORY.has(selectedModel.name)) {
      setThinking(true);
    }
  }, [selectedModel]);

  // Load models on mount
  useEffect(() => {
    const loadModels = async () => {
      try {
        const raw = await invoke("list_models");
        const parsed = (raw || []).map(([name, provider]) => ({
          name,
          provider: typeof provider === "string" ? provider : "Ollama",
          cloud: typeof provider === "string" ? provider === "OpenRouter" : false,
        }));
        setModels(parsed);
        if (parsed.length > 0) setSelectedModel(parsed[0]);
      } catch {
        addToast("Failed to load models", "error");
      }
    };
    loadModels();
  }, []);

  // Load chat list from DB on mount
  useEffect(() => {
    const loadChatList = async () => {
      try {
        const list = await invoke("load_chatlist");
        setChatList(list || []);
      } catch {
        // DB might not be ready yet, silently ignore
      }
    };
    loadChatList();
  }, []);

  // Save draft for current chat before switching
  const saveDraft = useCallback((chatId, text) => {
    if (!chatId) return;
    setDrafts((prev) => {
      if (!text && prev[chatId]) {
        const next = { ...prev };
        delete next[chatId];
        return next;
      }
      if (prev[chatId] === text) return prev;
      return { ...prev, [chatId]: text };
    });
  }, []);

  // Get draft for a chat
  const getDraft = useCallback((chatId) => {
    return drafts[chatId] || "";
  }, [drafts]);

  // Load messages when a chat is opened from sidebar
  const openChat = useCallback(async (id) => {
    if (id.startsWith("pending_")) {
      setActiveChatId(id);
      setActiveMessages([]);
      return;
    }

    setActiveChatId(id);
    setLoadingChat(true);
    setActiveMessages([]);
    try {
      const chat = await invoke("load_chat_memory", { id });
      const msgs = (chat.messages || []).map((m, i) => ({
        id: i,
        role: m.role === "assistant" ? "bot" : m.role,
        text: m.content,
      }));
      setActiveMessages(msgs);
    } catch (err) {
      const errMsg = typeof err === "string" ? err : err?.message || "Unknown error";
      addToast("Failed to load chat: " + errMsg, "error");
      setActiveMessages([]);
      setActiveChatId(null);
    } finally {
      setLoadingChat(false);
    }
  }, [addToast]);

  const getModelId = useCallback((model) => {
    if (model.cloud) return OPENROUTER_ID_MAP[model.name] || model.name;
    return model.name;
  }, []);

  // Start a new chat — add a temporary entry to sidebar, clear messages
  const startNewChat = useCallback(() => {
    const tempId = "pending_" + Date.now();
    setActiveChatId(tempId);
    setActiveMessages([]);
    setChatList((prev) => [
      { id: tempId, title: "New chat" },
      ...prev,
    ]);
  }, []);

  const refreshChatList = useCallback(async () => {
    try {
      const list = await invoke("load_chatlist");
      setChatList(list || []);
    } catch { /* ignore */ }
  }, []);

  // Send a message
  const send = useCallback(async (text, setInput) => {
    const trimmed = text.trim();
    if (!trimmed || isStreaming || !selectedModel) return;

    setInput("");
    setIsStreaming(true);

    const userMsg = { id: Date.now(), role: "user", text };
    const botMsg = { id: Date.now() + 1, role: "bot", text: "", model: selectedModel.name };

    setActiveMessages((prev) => [...prev, userMsg, botMsg]);

    const isPending = activeChatId && activeChatId.startsWith("pending_");

    try {
      const channel = new Channel();
      channel.onmessage = (token) => {
        setActiveMessages((prev) =>
          prev.map((m) =>
            m.id === botMsg.id ? { ...m, text: m.text + token } : m
          )
        );
      };

      const modelId = getModelId(selectedModel);
      const modelDetails = [modelId, selectedModel.provider];

      if (!isPending && activeChatId) {
        // Continue existing conversation from DB
        await invoke("continue_conversation", {
          id: activeChatId,
          message: trimmed,
          onEvent: channel,
          think: thinking,
          modelDetails,
        });
        refreshChatList();
      } else {
        // New chat — backend saves to DB and returns (chatId, fullResponse)
        const result = await invoke("new_chat", {
          message: trimmed,
          onEvent: channel,
          think: thinking,
          modelDetails,
        });
        if (result && result[0]) {
          const realId = result[0];
          setActiveChatId(realId);
          // Remove pending entry, keep user message as temp title until refreshChatList returns with AI title
          setChatList((prev) => {
            const cleaned = prev.filter((c) => !c.id.startsWith("pending_"));
            return [{ id: realId, title: trimmed.slice(0, 60) }, ...cleaned];
          });
          // refreshChatList will replace the temp title with AI-generated title from DB
          refreshChatList();
        }
      }
    } catch (err) {
      let errMsg = typeof err === "string" ? err : err?.message || "Unknown error";

      if (errMsg.includes("transformCallback")) {
        errMsg = 'Not running in Tauri. Use "npm run tauri dev" instead of "npm run dev".';
      } else if (errMsg.toLowerCase().includes("not found") || errMsg.toLowerCase().includes("no chat")) {
        errMsg += "\n\nThe chat may have been deleted. Starting a fresh conversation.";
        setActiveChatId(null);
      } else if (errMsg.toLowerCase().includes("connection") || errMsg.toLowerCase().includes("timeout")) {
        errMsg = "Could not reach the model. Make sure Ollama is running and the model is pulled.";
      }

      setActiveMessages((prev) =>
        prev.map((m) =>
          m.id === botMsg.id
            ? { ...m, text: m.text ? `${m.text}\n\nError: ${errMsg}` : `Error: ${errMsg}` }
            : m
        )
      );
    } finally {
      setIsStreaming(false);
      setActiveMessages((prev) =>
        prev.map((m) =>
          m.id === botMsg.id && !m.text
            ? { ...m, text: "No response received." }
            : m
        )
      );
    }
  }, [isStreaming, selectedModel, activeChatId, thinking, getModelId, addToast, refreshChatList]);

  return {
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
    refreshChatList,
    saveDraft,
    getDraft,
  };
}