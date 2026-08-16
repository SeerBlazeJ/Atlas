import { getThinkingStatus } from "../constants/models.js";

export default function Composer({ input, setInput, isStreaming, selectedModel, onSend, inputRef, thinking, onToggleThinking }) {
  const handleKey = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  };

  const canSend = input.trim() && !isStreaming && selectedModel;

  const thinkStatus = selectedModel ? getThinkingStatus(selectedModel.name) : "supported";
  const isMandatory = thinkStatus === "mandatory";

  // Auto-resize textarea without visual glitch
  const handleChange = (e) => {
    setInput(e.target.value);
    const ta = e.target;
    // Only recalc if content actually changed height
    const next = Math.min(ta.scrollHeight, 180) + "px";
    if (ta.style.height !== next) {
      ta.style.height = next;
    }
  };

  return (
    <div className="composer">
      <div className="composer-inner">
        <div className="input-row">
          <textarea
            ref={inputRef}
            placeholder="Message Atlas..."
            value={input}
            onChange={handleChange}
            onKeyDown={handleKey}
            rows={1}
          />
          <button
            className={`send-btn ${canSend ? "active" : ""}`}
            onClick={onSend}
            disabled={!canSend}
            title="Send"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
              <path d="M12 19V5M12 5l-6 6M12 5l6 6" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </div>
        <div className="input-meta">
          <div
            className="thinking-toggle"
            onClick={() => !isMandatory && onToggleThinking()}
            title={
              isMandatory
                ? "Thinking is required for this model"
                : "Toggle thinking mode"
            }
          >
            <div className={`toggle-switch ${thinking ? "on" : ""} ${isMandatory ? "mandatory" : ""}`}>
              <div className="toggle-knob" />
            </div>
            <span className={`thinking-label ${thinking ? "on" : ""}`}>
              Thinking
              {isMandatory && <span className="thinking-badge mandatory">Required</span>}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
