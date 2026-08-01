<script setup lang="ts">
import { computed, onMounted } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import Toast from "./components/ui/Toast.vue";
import { store } from "./store";

const route = useRoute();
const runningCount = computed(() => store.runningCount);
const pageTitle = computed(() => (route.meta.title as string | undefined) ?? "");

// 脚本表单 / 快速执行依赖 store.shells；Shell 管理页之外也需在启动时加载
onMounted(() => {
  void store.refreshShells();
});

interface NavItem {
  to: string;
  label: string;
  icon: string[];
}

const navItems: NavItem[] = [
  {
    to: "/scripts",
    label: "脚本库",
    icon: [
      "M7 3h10a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Z",
      "M9 8h6",
      "M9 12h6",
      "M9 16h4",
    ],
  },
  {
    to: "/console",
    label: "运行控制台",
    icon: [
      "M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z",
      "m7 9 3.5 3.5L7 16",
      "M13 16h4",
    ],
  },
  {
    to: "/history",
    label: "运行历史",
    icon: ["M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Z", "M12 7v5l3.5 2"],
  },
  {
    to: "/shells",
    label: "Shell 管理",
    icon: ["M8 6 4 12l4 6", "M16 6l4 6-4 6"],
  },
];
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <svg
          class="brand-mark"
          viewBox="0 0 24 24"
          width="22"
          height="22"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <rect x="4" y="3" width="16" height="18" rx="3" />
          <path d="m9 9 2.5 2.5L9 14" />
          <path d="M13 14h3" />
        </svg>
        <span class="brand-name">脚本管理</span>
      </div>

      <nav class="nav">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="nav-link"
          :title="item.label"
        >
          <svg
            class="nav-icon"
            viewBox="0 0 24 24"
            width="18"
            height="18"
            fill="none"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path v-for="(d, i) in item.icon" :key="i" :d="d" />
          </svg>
          <span class="nav-label">{{ item.label }}</span>
          <span
            v-if="item.to === '/console'"
            class="nav-badge"
            :class="{ 'is-active': runningCount > 0 }"
          >
            {{ runningCount }}
          </span>
        </RouterLink>
      </nav>
    </aside>

    <div class="main">
      <header class="topbar">
        <h1 class="page-title">{{ pageTitle }}</h1>
      </header>
      <section class="content">
        <RouterView v-slot="{ Component }">
          <KeepAlive>
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </section>
    </div>

    <Toast />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100%;
}

.sidebar {
  flex: none;
  width: 200px;
  display: flex;
  flex-direction: column;
  background: var(--bg-panel);
  border-right: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  height: 52px;
  padding: 0 var(--space-4);
  border-bottom: 1px solid var(--border);
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
}
.brand-name {
  font-size: var(--font-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--space-3) var(--space-2);
  overflow-y: auto;
}

.nav-link {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 0 12px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  border-left: 2px solid transparent;
  white-space: nowrap;
  overflow: hidden;
  transition: background-color 0.15s ease, color 0.15s ease;
}
.nav-link:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.nav-link.router-link-active {
  background: var(--bg-accent-soft);
  color: var(--accent);
  border-left-color: var(--accent);
  font-weight: 500;
}
.nav-icon {
  flex: none;
}
.nav-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-badge {
  flex: none;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  background: var(--border);
  color: var(--text-secondary);
  font-size: var(--font-xs);
  line-height: 18px;
  text-align: center;
}
.nav-badge.is-active {
  background: var(--accent);
  color: var(--accent-contrast);
  animation: badge-pulse 1.6s ease-in-out infinite;
}
@keyframes badge-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--status-running) 45%, transparent);
  }
  50% {
    box-shadow: 0 0 0 4px transparent;
  }
}

.main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.topbar {
  flex: none;
  height: 52px;
  display: flex;
  align-items: center;
  padding: 0 var(--space-5);
  border-bottom: 1px solid var(--border);
  background: var(--bg-panel);
}
.page-title {
  font-size: var(--font-lg);
  font-weight: 600;
}

.content {
  flex: 1;
  overflow: auto;
  padding: var(--space-5);
}

@media (max-width: 820px) {
  .sidebar {
    width: 56px;
  }
  .brand {
    justify-content: center;
    padding: 0;
  }
  .brand-name,
  .nav-label {
    display: none;
  }
  .nav-link {
    justify-content: center;
    padding: 0;
    border-left: none;
    border-top: 2px solid transparent;
  }
  .nav-link.router-link-active {
    border-left-color: transparent;
    border-top-color: var(--accent);
  }
  .nav-badge {
    position: absolute;
    top: 2px;
    right: 4px;
    min-width: 8px;
    width: 8px;
    height: 8px;
    padding: 0;
    font-size: 0;
    line-height: 8px;
  }
}
</style>
