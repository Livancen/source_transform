import { defineConfig, presetUno } from "unocss";

export default defineConfig({
  presets: [presetUno()],
  dark: "media",
  theme: {
    colors: {
      bg0: "#0e1117",
      bg1: "#151a22",
      bg2: "#1b2130",
      bg3: "#232a3b",
      "bg-hover": "#2a3348",
      "bg-selected": "rgba(91, 140, 255, 0.16)",
      border: "rgba(255, 255, 255, 0.07)",
      "border-strong": "rgba(255, 255, 255, 0.12)",
      t1: "#eef1f6",
      t2: "#9aa3b5",
      t3: "#6b7385",
      accent: "#5b8cff",
      "accent-soft": "rgba(91, 140, 255, 0.14)",
      "accent-hover": "#7aa3ff",
      success: "#34d399",
      "success-soft": "rgba(52, 211, 153, 0.12)",
      danger: "#f87171",
      image: "#a78bfa",
      video: "#38bdf8",
    },
  },
  shortcuts: {
    "app-shell": "h-full min-h-0 flex flex-col",
    "tb-btn":
      "inline-flex items-center gap-6px h-34px px-11px border border-transparent rounded-6px bg-transparent text-t2 text-12px font-500 whitespace-nowrap transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg3 hover:not-disabled:text-t1 hover:not-disabled:border-border disabled:opacity-45 disabled:cursor-not-allowed",
    "tb-btn-active":
      "bg-accent-soft text-accent-hover border-[rgba(91,140,255,0.3)]",
    "tb-btn-success":
      "bg-gradient-to-br from-#34d399 to-#10b981 text-#042f1e font-600 border-transparent shadow-[0_2px_10px_rgba(52,211,153,0.28)] hover:not-disabled:brightness-105 hover:not-disabled:text-#042f1e hover:not-disabled:border-transparent",
    "modal-mask":
      "fixed inset-0 bg-black/55 backdrop-blur-4px flex items-center justify-center z-1000 p-24px",
    "modal-panel":
      "bg-bg1 border border-border-strong rounded-14px shadow-[0_24px_64px_rgba(0,0,0,0.5)] color-t1 max-w-94% max-h-92% overflow-auto",
    "modal-btn":
      "inline-flex items-center justify-center h-34px px-14px rounded-6px border border-border bg-bg3 color-t1 text-12px font-500 transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg-hover hover:not-disabled:border-border-strong disabled:opacity-45 disabled:cursor-not-allowed",
    "modal-btn-primary":
      "bg-gradient-to-br from-#5b8cff to-#4f7af0 border-transparent color-white shadow-[0_2px_10px_rgba(91,140,255,0.3)] hover:not-disabled:brightness-110 hover:not-disabled:border-transparent hover:not-disabled:color-white",
    field:
      "h-30px px-10px rounded-7px border border-border bg-bg0 color-t1 text-12px outline-none focus:border-accent",
    "opt-chip":
      "inline-flex items-center gap-8px min-h-34px py-4px pr-10px pl-8px rounded-8px bg-bg2 border border-border transition-all duration-150",
    "opt-chip-on":
      "border-[rgba(91,140,255,0.4)] bg-accent-soft",
  },
});
