// Only this temporary, top-level login document is observed.
(() => {
  if (window !== window.top || location.origin !== "https://discord.com") return;
  const key = "__RUSTCORD_CAPABILITY__";
  const started = Date.now();
  let candidate = null;
  let finished = false;
  // The engine's channel to Rustcord, taken before the page's own scripts run:
  // WebView2's on Windows, WebKit's handler on Linux.
  const channel = window.chrome?.webview ?? window.webkit?.messageHandlers?.runeLoginWake;
  if (!channel) return;
  const wake = channel.postMessage.bind(channel);
  const active = () => !finished && Date.now() - started < 600000 &&
    window === window.top && location.origin === "https://discord.com";
  const apiRequest = address => {
    try {
      const parsed = new URL(address, location.href);
      return parsed.origin === "https://discord.com" && !parsed.username &&
        !parsed.password && /^\/api\/v[0-9]+\//.test(parsed.pathname);
    } catch { return false; }
  };
  // The answers that carry the new session: the password's and MFA's.
  const sessionAnswer = address => {
    try {
      return apiRequest(address) && /^\/api\/v[0-9]+\/auth\/(login|mfa\/[a-z]+)$/
        .test(new URL(address, location.href).pathname);
    } catch { return false; }
  };
  const tokenIn = body => typeof body?.token === "string" ? body.token : null;
  // Taken before the page's own scripts can hide it. Once signed in the web
  // client stores the session here and moves to the app in this document:
  // the session is taken then, before the app (heavy for the engine) loads.
  let storage = null;
  try { storage = window.localStorage; } catch { /* No storage: the requests still tell. */ }
  const stored = () => {
    try {
      const value = storage?.getItem("token");
      return value ? JSON.parse(value) : null;
    } catch { return null; }
  };
  const observe = value => {
    if (!active() || candidate !== null || typeof value !== "string" ||
        value.length < 16 || value.length > 2048 || !/^[\x21-\x7e]+$/.test(value)) return;
    candidate = value;
    // This IPC carries no credential. The native side queries the main frame.
    wake(true);
  };
  Object.defineProperty(window, "__rustcord_take_" + key, {
    configurable: false, writable: false,
    value: () => {
      if (!active()) { candidate = null; finished = true; return null; }
      if (candidate === null) return null;
      const result = key + ":" + candidate;
      candidate = null;
      finished = true;
      return result;
    }
  });
  for (const name of ["pushState", "replaceState"]) {
    const original = history[name];
    history[name] = function (...args) {
      observe(stored());
      return Reflect.apply(original, this, args);
    };
  }
  const targets = new WeakMap();
  const answers = new WeakSet();
  const xhrOpen = XMLHttpRequest.prototype.open;
  const xhrHeader = XMLHttpRequest.prototype.setRequestHeader;
  const xhrSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function (...args) {
    targets.set(this, apiRequest(args[1]));
    if (sessionAnswer(args[1])) answers.add(this); else answers.delete(this);
    return Reflect.apply(xhrOpen, this, args);
  };
  XMLHttpRequest.prototype.send = function (...args) {
    if (answers.has(this)) {
      this.addEventListener("load", () => {
        try {
          observe(tokenIn(this.responseType === "json" ? this.response : JSON.parse(this.responseText)));
        } catch { /* Not JSON: no session in it. */ }
      });
    }
    return Reflect.apply(xhrSend, this, args);
  };
  XMLHttpRequest.prototype.setRequestHeader = function (...args) {
    if (targets.get(this) && String(args[0]).toLowerCase() === "authorization") observe(args[1]);
    return Reflect.apply(xhrHeader, this, args);
  };
  const fetch = window.fetch;
  window.fetch = function (input, options) {
    try {
      const request = input instanceof Request;
      if (apiRequest(request ? input.url : input)) {
        const headers = new Headers(options?.headers ?? (request ? input.headers : undefined));
        observe(headers.get("authorization"));
      }
    } catch { /* Observation must not change the browser request's behavior. */ }
    const answer = Reflect.apply(fetch, this, arguments);
    try {
      if (sessionAnswer(input instanceof Request ? input.url : input)) {
        answer.then(response => response.clone().json())
          .then(body => observe(tokenIn(body)), () => {});
      }
    } catch { /* As above. */ }
    return answer;
  };
})();
