import { defineConfig, presetUno } from "unocss";

/**
 * Fluent 2 视觉 token（方案 A：不换库，只换观感）
 * 参考：Fluent 2 light theme — brand / neutral / stroke / shadow
 */
export default defineConfig({
  presets: [presetUno()],
  dark: "media",
  theme: {
    colors: {
      // 层级背景（Fluent neutral background）
      bg0: "#f5f5f5",
      bg1: "#ffffff",
      bg2: "#fafafa",
      bg3: "#f0f0f0",
      "bg-hover": "#f5f5f5",
      "bg-selected": "rgba(15, 108, 189, 0.1)",
      // 描边
      border: "#e0e0e0",
      "border-strong": "#c7c7c7",
      // 文字
      t1: "#242424",
      t2: "#616161",
      t3: "#707070",
      // 主操作（Fluent brand）
      primary: "#0f6cbd",
      "primary-hover": "#115ea3",
      "primary-pressed": "#0c3b5e",
      "primary-soft": "rgba(15, 108, 189, 0.12)",
      // 次要 / 强调（与 brand 同源）
      secondary: "#0f6cbd",
      "secondary-hover": "#115ea3",
      "secondary-soft": "rgba(15, 108, 189, 0.1)",
      // 取消 / 中性按钮底
      cancel: "#f5f5f5",
      "cancel-hover": "#ebebeb",
      accent: "#0f6cbd",
      "accent-soft": "rgba(15, 108, 189, 0.1)",
      "accent-hover": "#115ea3",
      // 语义色
      success: "#0e700e",
      "success-soft": "rgba(14, 112, 14, 0.1)",
      danger: "#c50f1f",
      "danger-soft": "rgba(197, 15, 31, 0.1)",
      warning: "#8a3707",
      "warning-soft": "rgba(138, 55, 7, 0.1)",
      // 类型标签
      image: "#5c2e91",
      video: "#0f6cbd",
    },
    borderRadius: {
      fluent: "4px",
      "fluent-lg": "8px",
      "fluent-xl": "12px",
    },
    boxShadow: {
      "fluent-sm": "0 1px 2px rgba(0, 0, 0, 0.14), 0 0 2px rgba(0, 0, 0, 0.12)",
      fluent: "0 2px 4px rgba(0, 0, 0, 0.14), 0 0 2px rgba(0, 0, 0, 0.12)",
      "fluent-md":
        "0 4px 8px rgba(0, 0, 0, 0.14), 0 0 2px rgba(0, 0, 0, 0.12)",
      "fluent-lg":
        "0 8px 16px rgba(0, 0, 0, 0.14), 0 0 2px rgba(0, 0, 0, 0.12)",
    },
  },
  shortcuts: {
    "app-shell": "h-full min-h-0 flex flex-col",
    "tb-btn":
      "inline-flex items-center gap-6px h-32px px-12px border border-transparent rounded-4px bg-transparent text-t2 text-12px font-500 whitespace-nowrap transition-all duration-100 cursor-pointer hover:not-disabled:bg-bg0 hover:not-disabled:text-t1 disabled:opacity-40 disabled:cursor-not-allowed",
    "tb-btn-active":
      "bg-secondary-soft text-secondary border-transparent",
    "tb-btn-success":
      "bg-primary text-white font-600 border-transparent hover:not-disabled:bg-primary-hover hover:not-disabled:text-white hover:not-disabled:border-transparent shadow-fluent-sm",
    "modal-mask":
      "fixed inset-0 bg-black/40 flex items-center justify-center z-1000 p-24px",
    "modal-panel":
      "relative bg-bg1 border border-border rounded-8px shadow-fluent-lg color-t1 max-w-94% max-h-92% overflow-auto",
    "modal-close":
      "absolute top-12px right-12px w-32px h-32px grid place-items-center rounded-4px border-none bg-transparent color-t3 cursor-pointer transition-all duration-100 hover:not-disabled:bg-bg0 hover:not-disabled:color-t1 disabled:opacity-40 disabled:cursor-not-allowed",
    "modal-btn":
      "inline-flex items-center justify-center h-32px px-16px rounded-4px border border-border-strong bg-bg1 color-t1 text-12px font-500 transition-all duration-100 cursor-pointer hover:not-disabled:bg-bg0 disabled:opacity-40 disabled:cursor-not-allowed",
    "modal-btn-primary":
      "bg-primary border-transparent color-white font-600 shadow-fluent-sm hover:not-disabled:bg-primary-hover hover:not-disabled:border-transparent hover:not-disabled:color-white",
    "modal-btn-secondary":
      "bg-bg1 border border-border-strong color-t1 font-500 hover:not-disabled:bg-bg0 hover:not-disabled:border-border-strong hover:not-disabled:color-t1",
    "modal-btn-cancel":
      "bg-cancel border border-border-strong color-t1 font-500 hover:not-disabled:bg-cancel-hover",
    field:
      "h-32px px-10px rounded-4px border border-border-strong bg-bg1 color-t1 text-12px outline-none transition-colors duration-100 focus:border-primary focus:shadow-[0_0_0_1px_#0f6cbd]",
    "opt-chip":
      "inline-flex items-center gap-8px min-h-32px py-4px pr-10px pl-8px rounded-4px bg-bg0 border border-border transition-all duration-100",
    "opt-chip-on":
      "border-primary/35 bg-primary-soft",
    "fluent-pivot-item":
      "relative h-full px-14px text-12px font-500 border-none bg-transparent color-t2 cursor-pointer transition-colors duration-100 hover:not-disabled:color-t1 hover:not-disabled:bg-bg0 disabled:opacity-40 disabled:cursor-not-allowed",
  },
});
