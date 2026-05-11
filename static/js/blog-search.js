(function () {
  function attach(input) {
    if (!input || input.dataset.suggestInit === '1') return;
    input.dataset.suggestInit = '1';

    const suggestUrl = input.dataset.suggestUrl || '/blog/search/suggest';
    const dropdown = document.getElementById(input.id === 'sidebarSearch' ? 'sidebarSuggest' : 'mainSuggest');
    if (!dropdown) return;

    let timer = null;
    let lastQuery = '';
    let abortCtl = null;

    const hide = () => {
      dropdown.hidden = true;
      dropdown.innerHTML = '';
    };

    const fetchSuggest = (q) => {
      if (abortCtl) abortCtl.abort();
      abortCtl = new AbortController();
      fetch(suggestUrl + '?q=' + encodeURIComponent(q), { signal: abortCtl.signal })
        .then((r) => (r.ok ? r.text() : ''))
        .then((html) => {
          if (!html) return hide();
          dropdown.innerHTML = html;
          dropdown.hidden = false;
        })
        .catch(() => {});
    };

    input.addEventListener('input', () => {
      const q = input.value.trim();
      if (timer) clearTimeout(timer);
      if (q === lastQuery) return;
      lastQuery = q;
      if (q.length < 1) {
        hide();
        return;
      }
      timer = setTimeout(() => fetchSuggest(q), 160);
    });

    input.addEventListener('focus', () => {
      if (input.value.trim() && dropdown.innerHTML) dropdown.hidden = false;
    });

    document.addEventListener('click', (e) => {
      if (e.target === input) return;
      if (dropdown.contains(e.target)) return;
      hide();
    });

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') hide();
    });
  }

  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('input[data-suggest-url]').forEach(attach);
  });
})();
