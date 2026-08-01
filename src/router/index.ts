import { createRouter, createWebHashHistory } from "vue-router";
import HistoryView from "../components/HistoryView.vue";
import RunConsoleView from "../components/RunConsoleView.vue";
import ScriptsView from "../components/ScriptsView.vue";
import ShellManagementView from "../components/ShellManagementView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/scripts" },
    { path: "/scripts", component: ScriptsView, meta: { title: "脚本库" } },
    { path: "/console", component: RunConsoleView, meta: { title: "运行控制台" } },
    { path: "/history", component: HistoryView, meta: { title: "运行历史" } },
    { path: "/shells", component: ShellManagementView, meta: { title: "Shell 管理" } },
  ],
});
