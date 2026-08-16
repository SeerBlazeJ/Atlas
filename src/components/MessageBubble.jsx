import { BotMessage } from "./MarkdownRenderer";

export default function MessageBubble({ msg }) {
  if (msg.role === "user") {
    return (
      <div className="message user">
        <div className="msg-content">
          <div className="msg-text user-bubble">{msg.text}</div>
        </div>
      </div>
    );
  }

  return (
    <div className="message bot">
      <div className="msg-avatar">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" />
        </svg>
      </div>
      <div className="msg-content">
        <div className="msg-label">{msg.model || "Atlas"}</div>
        {msg.text === "" ? (
          <div className="typing-dots"><span /><span /><span /></div>
        ) : (
          <div className="msg-text"><BotMessage text={msg.text} /></div>
        )}
        {msg.text && !msg.text.startsWith("Error:") && (
          <div className="msg-actions">
            <button className="msg-action" onClick={() => navigator.clipboard?.writeText(msg.text)} title="Copy">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
                <rect x="9" y="9" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.8" />
                <path d="M5 15V5a2 2 0 012-2h10" stroke="currentColor" strokeWidth="1.8" />
              </svg>
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
