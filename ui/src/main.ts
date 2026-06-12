// 应用入口：Vue + Pinia + Vue Router 初始化，路由配置

import { createApp } from "vue";
import { createPinia } from "pinia";
import { createRouter, createWebHistory } from "vue-router";
import App from "./App.vue";
import "./styles/main.css";

// 路由配置
const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/login" },
    { path: "/login", component: () => import("./views/LoginView.vue") },
    { path: "/diagnosis", component: () => import("./views/DiagnosisView.vue") },
    { path: "/services", component: () => import("./views/ServicesView.vue") },
    { path: "/convert", component: () => import("./views/ConvertView.vue") },
    { path: "/qzone", component: () => import("./views/QzoneView.vue") },
    { path: "/whiteboard", component: () => import("./views/WhiteboardView.vue") },
    { path: "/cleaner", component: () => import("./views/CleanerView.vue") },
    { path: "/course", component: () => import("./views/CourseView.vue") },
    { path: "/pet", component: () => import("./views/PetView.vue") },
    { path: "/more", component: () => import("./views/MoreView.vue") },
  ],
});

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
