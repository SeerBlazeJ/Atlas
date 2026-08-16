import { useState } from "react";

export default function Sidebar({ chatList, activeChatId, onOpenChat, onNewChat, open, onClose }) {
  const [search, setSearch] = useState("");

  const filtered = chatList.filter((c) =>
    c.title.toLowerCase().includes(search.toLowerCase())
  );

  const grouped = {};
  filtered.forEach((c) => {
    // Use current time for grouping since DB may not have createdAt
    const g = "All Chats";
    (grouped[g] ||= []).push(c);
  });

  return (
    <>
      {open && (
        <div className="sidebar-overlay" onClick={onClose} />
      )}
      <aside className={`sidebar ${open ? "open" : ""}`}>
        <div className="sidebar-header">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" />
          </svg>
          Atlas
        </div>
        <button className="new-chat-btn" onClick={() => { onNewChat(); onClose(); }}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M12 5v14M5 12h14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
          </svg>
          New chat
        </button>
        <div className="search-box">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
            <circle cx="11" cy="11" r="7" stroke="currentColor" strokeWidth="2" />
            <path d="M21 21l-4.3-4.3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
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
                  className={`history-item ${c.id === activeChatId ? "active" : ""}`}
                  onClick={() => { onOpenChat(c.id); onClose(); }}
                >
                  <svg className="history-item-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                  <span className="history-item-title">{c.title}</span>
                </div>
              ))}
            </div>
          ))}
          {filtered.length === 0 && chatList.length > 0 && (
            <div className="history-empty">
              No results found
            </div>
          )}
          {chatList.length === 0 && (
            <div className="history-empty">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" style={{ opacity: 0.3, marginBottom: "8px" }}>
                <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
              No conversations yet
            </div>
          )}
        </div>
      </aside>
    </>
  );
}
