import { useRef, useEffect, useState } from "react";

export default function TopBar({ selectedModel, models, onSelectModel, onToggleSidebar }) {
  return (
    <div className="topbar">
      <button className="menu-toggle" onClick={onToggleSidebar}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M4 6h16M4 12h16M4 18h16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </svg>
      </button>

      <div className="topbar-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="topbar-logo">
          <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" />
        </svg>
        <span>Atlas</span>
      </div>

      <div className="topbar-spacer" />

      <ModelSelectorInline models={models} selectedModel={selectedModel} onSelect={onSelectModel} />
    </div>
  );
}

function ModelSelectorInline({ models, selectedModel, onSelect }) {
  const [open, setOpen] = useState(false);
  const ref = useRef(null);

  useEffect(() => {
    function outside(e) {
      if (open && ref.current && !ref.current.contains(e.target)) setOpen(false);
    }
    document.addEventListener("mousedown", outside);
    return () => document.removeEventListener("mousedown", outside);
  }, [open]);

  return (
    <div ref={ref} className={`model-selector compact ${open ? "open" : ""}`}>
      <button className="model-trigger compact" onClick={() => setOpen(!open)}>
        {selectedModel?.cloud ? (
          <svg className="model-type-icon" width="13" height="13" viewBox="0 0 24 24" fill="none">
            <path d="M6.5 19a4.5 4.5 0 01-.4-8.98A5.5 5.5 0 0116.5 8.5a4 4 0 01-.5 7.97" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        ) : (
          <svg className="model-type-icon" width="13" height="13" viewBox="0 0 24 24" fill="none">
            <rect x="2" y="3" width="20" height="14" rx="2" stroke="currentColor" strokeWidth="2" />
            <path d="M8 21h8M12 17v4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
          </svg>
        )}
        <span className="model-name">{selectedModel ? selectedModel.name : "Select model"}</span>
        <svg className="model-chevron" width="11" height="11" viewBox="0 0 24 24" fill="none">
          <path d="M6 9l6 6 6-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>
      <div className="model-dropdown">
        {models.length === 0 && <div className="model-dropdown-empty">No models found</div>}
        {models.map((m) => (
          <div
            key={m.name + m.provider}
            className={`model-option ${selectedModel?.name === m.name && selectedModel?.provider === m.provider ? "selected" : ""}`}
            onClick={() => { onSelect(m); setOpen(false); }}
          >
            <div className="model-option-info">
              <div className="model-option-name">
                <span className="model-option-name-text">{m.name}</span>
                {m.cloud ? (
                  <svg className="model-type-icon-sm" width="12" height="12" viewBox="0 0 24 24" fill="none">
                    <path d="M6.5 19a4.5 4.5 0 01-.4-8.98A5.5 5.5 0 0116.5 8.5a4 4 0 01-.5 7.97" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                ) : (
                  <svg className="model-type-icon-sm" width="12" height="12" viewBox="0 0 24 24" fill="none">
                    <rect x="2" y="3" width="20" height="14" rx="2" stroke="currentColor" strokeWidth="2" />
                    <path d="M8 21h8M12 17v4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                  </svg>
                )}
                {m.cloud && <span className="model-dot online" />}
              </div>
              <div className="model-option-provider">{m.provider}</div>
            </div>
            <svg className="model-option-check" width="14" height="14" viewBox="0 0 24 24" fill="none">
              <path d="M5 13l4 4L19 7" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </div>
        ))}
      </div>
    </div>
  );
}
