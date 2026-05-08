import { createApp } from "vue";
import { createPinia } from "pinia";
import { createI18n } from "vue-i18n";
import App from "./App.vue";
import router from "./router";

// Tailwind CSS + daisyUI + global CSS variables
import "./assets/main.css";

// i18n
import zhCN from "./locales/zh-CN";
import en from "./locales/en";

function getDefaultLocale(): "zh-CN" | "en" {
  try {
    const saved = localStorage.getItem("locale");
    if (saved && ["zh-CN", "en"].includes(saved)) return saved as "zh-CN" | "en";
  } catch {}
  const lang = navigator.language;
  if (lang.startsWith("zh")) return "zh-CN";
  return "en";
}

const i18n = createI18n({
  legacy: false,
  locale: getDefaultLocale(),
  fallbackLocale: "en",
  messages: {
    "zh-CN": zhCN,
    en: en,
  },
});

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(i18n);

// Global error handler
window.onerror = function (msg, url, line, col, error) {
  console.error("[GlobalError]", msg, `Line: ${line}, Col: ${col}`, error?.stack || "");
  return false;
};
window.addEventListener("unhandledrejection", function (e) {
  console.error("[UnhandledRejection]", e.reason?.message || e.reason || String(e), e.reason?.stack || "");
});

// Tauri system logger
const prefixRe = /^\[([^\]]+)\]\s*/;
const logQueue: Array<{ level: string; prefix: string; message: string }> = [];
let flushTimer: ReturnType<typeof setTimeout> | null = null;

const flushLogs = async () => {
  if (logQueue.length === 0) return;
  const batch = logQueue.splice(0);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    for (const entry of batch) {
      try {
        await invoke("write_system_log", {
          level: entry.level,
          prefix: entry.prefix,
          message: entry.message,
        });
      } catch {}
    }
  } catch {}
};

const scheduleFlush = () => {
  if (flushTimer) return;
  flushTimer = setTimeout(() => {
    flushTimer = null;
    flushLogs();
  }, 500);
};

const queueLog = (level: string, args: any[]) => {
  const message = args
    .map((a) => {
      if (typeof a === "string") return a;
      try { return JSON.stringify(a); } catch { return String(a); }
    })
    .join(" ");
  const match = message.match(prefixRe);
  const prefix = match ? match[1] : "App";
  const cleanMsg = match ? message.slice(match[0].length) : message;
  const truncated = cleanMsg.length > 2000 ? cleanMsg.slice(0, 2000) + "..." : cleanMsg;
  logQueue.push({ level, prefix, message: truncated });
  scheduleFlush();
};

const origLog = console.log.bind(console);
const origWarn = console.warn.bind(console);
const origError = console.error.bind(console);

console.log = function (...args: any[]) {
  origLog(...args);
  try { queueLog("info", args); } catch {};
};
console.warn = function (...args: any[]) {
  origWarn(...args);
  try { queueLog("warn", args); } catch {};
};
console.error = function (...args: any[]) {
  origError(...args);
  try { queueLog("error", args); } catch {};
};

window.addEventListener("beforeunload", () => {
  if (logQueue.length > 0) flushLogs();
});

// Mount
try {
  app.mount("#app");
} catch (e: unknown) {
  const msg = e instanceof Error ? `${e.message}\n${e.stack}` : String(e);
  console.error("[Vue mount failed]", msg);
  const el = document.getElementById("app");
  if (el) {
    el.innerHTML = `<div style="color:#e55;padding:20px;font-family:monospace;font-size:13px;white-space:pre-wrap">
<strong>⚠️ Vue mount failed</strong>
${msg}
</div>`;
  }
}
