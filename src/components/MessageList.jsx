import { useRef, useEffect } from "react";
import MessageBubble from "./MessageBubble";

export default function MessageList({ messages, isStreaming, loadingChat, endRef }) {
  const innerRef = useRef(null);

  return (
    <div className="messages">
      <div className="messages-inner" ref={innerRef}>
        {loadingChat ? (
          <div className="chat-loading">
            <div className="loading-spinner" />
            <span>Loading conversation...</span>
          </div>
        ) : messages.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <svg width="40" height="40" viewBox="0 0 24 24" fill="none">
                <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
              </svg>
            </div>
            <h2>What can I help with?</h2>
            <p>Start a conversation with Atlas.</p>
          </div>
        ) : (
          messages.map((msg) => <MessageBubble key={msg.id} msg={msg} />)
        )}
        <div ref={endRef} />
      </div>
    </div>
  );
}
