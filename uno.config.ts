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
      "bg-selected": "rgba(0, 122, 204, 0.08)",
      border: "#eeeeee",
      "border-strong": "#e0e0e0",
      t1: "#1f2329",
      t2: "#646a73",
      t3: "#8f959e",
      // 主操作 / 确定
      primary: "#04d76a",
      "primary-hover": "#03c25f",
      // 次要
      secondary: "#007acc",
      "secondary-hover": "#006bb3",
      "secondary-soft": "rgba(0, 122, 204, 0.08)",
      // 取消
      cancel: "#f4f5f7",
      "cancel-hover": "#e8eaed",
      accent: "#007acc",
      "accent-soft": "rgba(0, 122, 204, 0.08)",
      "accent-hover": "#006bb3",
      success: "#04d76a",
      "success-soft": "rgba(4, 215, 106, 0.1)",
      danger: "#ff4d4f",
      image: "#722ed1",
      video: "#007acc",
    },
  },
  shortcuts: {
    "app-shell": "h-full min-h-0 flex flex-col",
    "tb-btn":
      "inline-flex items-center gap-6px h-34px px-11px border border-transparent rounded-6px bg-transparent text-t2 text-12px font-500 whitespace-nowrap transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg0 hover:not-disabled:text-t1 hover:not-disabled:border-border disabled:opacity-45 disabled:cursor-not-allowed",
    "tb-btn-active":
      "bg-secondary-soft text-secondary border-secondary/20",
    "tb-btn-success":
      "bg-primary text-white font-600 border-transparent hover:not-disabled:bg-primary-hover hover:not-disabled:text-white hover:not-disabled:border-transparent",
    "modal-mask":
      "fixed inset-0 bg-black/35 backdrop-blur-2px flex items-center justify-center z-1000 p-24px",
    "modal-panel":
      "relative bg-bg1 border border-border rounded-12px shadow-[0_8px_28px_rgba(0,0,0,0.12)] color-t1 max-w-94% max-h-92% overflow-auto",
    "modal-close":
      "absolute top-12px right-12px w-32px h-32px grid place-items-center rounded-6px border-none bg-transparent color-t3 cursor-pointer transition-all duration-150 hover:not-disabled:bg-bg0 hover:not-disabled:color-t1 disabled:opacity-45 disabled:cursor-not-allowed",
    "modal-btn":
      "inline-flex items-center justify-center h-34px px-14px rounded-6px border border-border bg-bg1 color-t1 text-12px font-500 transition-all duration-150 cursor-pointer hover:not-disabled:bg-bg0 hover:not-disabled:border-border-strong disabled:opacity-45 disabled:cursor-not-allowed",
    "modal-btn-primary":
      "bg-primary border-transparent color-white font-600 hover:not-disabled:bg-primary-hover hover:not-disabled:border-transparent hover:not-disabled:color-white",
    "modal-btn-secondary":
      "bg-secondary border-transparent color-white font-500 hover:not-disabled:bg-secondary-hover hover:not-disabled:border-transparent hover:not-disabled:color-white",
    "modal-btn-cancel":
      "bg-cancel border border-border color-t1 font-500 hover:not-disabled:bg-cancel-hover hover:not-disabled:border-border-strong",
    field:
      "h-30px px-10px rounded-6px border border-border bg-bg0 color-t1 text-12px outline-none focus:border-secondary focus:bg-bg1",
    "opt-chip":
      "inline-flex items-center gap-8px min-h-34px py-4px pr-10px pl-8px rounded-8px bg-bg0 border border-border transition-all duration-150",
    "opt-chip-on":
      "border-secondary/30 bg-secondary-soft",
  },
});
