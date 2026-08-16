import { createApp } from "vue";
import "virtual:uno.css";
import "./styles/base.less";
import App from "./App.vue";
import router from "./router";

createApp(App).use(router).mount("#app");
