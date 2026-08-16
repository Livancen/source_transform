import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import SettingsView from "../views/SettingsView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
      meta: { title: "素材转换", keepAlive: true },
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView,
      meta: { title: "设置", keepAlive: true },
    },
  ],
});

export default router;
