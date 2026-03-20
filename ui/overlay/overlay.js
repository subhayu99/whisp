const { invoke } = window.__TAURI__.core;

const container = document.getElementById('suggestion-container');
const textEl = document.getElementById('suggestion-text');

// Listen for suggestion events from the backend
window.__TAURI__.event.listen('show-suggestion', (event) => {
  const { text } = event.payload;
  textEl.textContent = text;
  container.classList.remove('hidden');
});

window.__TAURI__.event.listen('hide-suggestion', () => {
  container.classList.add('hidden');
  textEl.textContent = '';
});

// Keyboard shortcuts handled by the Rust backend (global hooks)
// This overlay is just a display layer
