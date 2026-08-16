import { defineConfig, presetUno } from "unocss";

export default defineConfig({
  presets: [presetUno()],
  dark: "media",
  theme: {
    colors: {
      bg0: "#f1f2f3",
      bg1: "#ffffff",
      bg2: "#ffffff",
      bg3: "#f7f8f9",
      "bg-hover": "#f1f2f3",
      "bg-selected": "rgba(22, 119, 255, 0.08)",
      border: "#eeeeee",
      "border-strong": "#e0e0e0",
      t1: "#1f2329",
      t2: "#646a73",
      t3: "#8f959e",
      accent: "#1677ff",
      "accent-soft": "rgba(22, 119, 255, 0.08)",
      "accent-hover": "#4096ff",
      success: "#52c41a",
      "success-soft": "rgba(82, 196, 26, 0.1)",
      danger: "#ff4d4f",
      image: "#722ed1",
      video: "#1677ff",
    },
  },
  shortcuts: {
    "app-shell": "h-full min-h-0 flex flex-col",
    "tb-btn":
      "inline-flex items-center gap-6px h-34px px-11px border border-transparent rounded-6px bg-transparent text-t2 text-12px font-500 whitespace-nowrap transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg0 hover:not-disabled:text-t1 hover:not-disabled:border-border disabled:opacity-45 disabled:cursor-not-allowed",
    "tb-btn-active":
      "bg-accent-soft text-accent border-accent/20",
    "tb-btn-success":
      "bg-accent text-white font-600 border-transparent hover:not-disabled:bg-accent-hover hover:not-disabled:text-white hover:not-disabled:border-transparent",
    "modal-mask":
      "fixed inset-0 bg-black/35 backdrop-blur-2px flex items-center justify-center z-1000 p-24px",
    "modal-panel":
      "bg-bg1 border border-border rounded-12px shadow-[0_8px_28px_rgba(0,0,0,0.12)] color-t1 max-w-94% max-h-92% overflow-auto",
    "modal-btn":
      "inline-flex items-center justify-center h-34px px-14px rounded-6px border border-border bg-bg1 color-t1 text-12px font-500 transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg0 hover:not-disabled:border-border-strong disabled:opacity-45 disabled:cursor-not-allowed",
    "modal-btn-primary":
      "bg-accent border-transparent color-white font-600 hover:not-disabled:bg-accent-hover hover:not-disabled:border-transparent hover:not-disabled:color-white",
    field:
      "h-30px px-10px rounded-6px border border-border bg-bg0 color-t1 text-12px outline-none focus:border-accent focus:bg-bg1",
    "opt-chip":
      "inline-flex items-center gap-8px min-h-34px py-4px pr-10px pl-8px rounded-8px bg-bg0 border border-border transition-all duration-150",
    "opt-chip-on":
      "border-accent/30 bg-accent-soft",
  },
});
