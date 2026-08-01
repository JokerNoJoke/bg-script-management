<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { statusMeta, store } from "../store";
import type { LiveRun } from "../store";
import Button from "./ui/Button.vue";
import Confirm from "./ui/Confirm.vue";
import LogConsole from "./LogConsole.vue";

const props = defineProps<{ live: LiveRun }>();

const rec = computed(() => props.live.record);
const running = computed(() => rec.value.status === "running");

// 运行中每秒 tick 刷新时长
const now = ref(Date.now());
let timer: number | undefined;
onMounted(() => {
  if (running.value) timer = window.setInterval(() => (now.value = Date.now()), 1000);
});
watch(running, (isRunning) => {
  if (isRunning) {
    if (timer === undefined) timer = window.setInterval(() => (now.value = Date.now()), 1000);
  } else {
    window.clearInterval(timer);
    timer = undefined;
  }
});
onBeforeUnmount(() => window.clearInterval(timer));

const duration = computed(() => {
  const end = rec.value.finishedAt ?? now.value;
  const sec = Math.max(0, Math.floor((end - rec.value.startedAt) / 1000));
  return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}`;
});

const source = computed(() => (rec.value.scriptId ? "脚本库" : "快速执行"));

const statusTone = computed(() => statusMeta[rec.value.status].tone);

const statusVar: Record<string, string> = {
  running: "var(--status-running)",
  success: "var(--status-success)",
  failed: "var(--status-failed)",
  timeout: "var(--status-timeout)",
  killed: "var(--status-killed)",
  interrupted: "var(--status-interrupted)",
  neutral: "var(--text-tertiary)",
};
const dotColor = computed(() => statusVar[statusTone.value]);

// 终止
const confirmOpen = ref(false);
const killing = ref(false);
async function kill() {
  if (killing.value) return;
  killing.value = true;
  await store.killRun(rec.value.id);
  killing.value = false;
  confirmOpen.value = false;
}

// 收起（仅已结束卡片）
function toggleCollapse() {
  props.live.collapsed = !props.live.collapsed;
}

// 结束态结果行
const result = computed<{ text: string; cls: string } | null>(() => {
  switch (rec.value.status) {
    case "success":
      return { text: `✓ 退出码 ${rec.value.exitCode ?? 0}`, cls: "ok" };
    case "failed":
      return { text: `✗ 退出码 ${rec.value.exitCode ?? "?"}`, cls: "bad" };
    case "timeout":
      return { text: "⏱ 已超时", cls: "timeout" };
    case "killed":
      return { text: "✕ 已终止", cls: "killed" };
    default:
      return null;
  }
});
</script>

<template>
  <section class="run-card">
    <header class="card-head">
      <span class="status-dot" :class="{ 'is-running': running }" :style="{ backgroundColor: dotColor }" />
      <h4 class="card-title" :title="rec.scriptName">{{ rec.scriptName }}</h4>
      <span v-if="running" class="card-running">运行中</span>
      <span v-else class="card-status" :style="{ color: dotColor }">{{ statusMeta[rec.status].label }}</span>
      <span class="card-dur mono">{{ duration }}</span>
      <div class="card-actions">
        <Button v-if="running" size="sm" variant="danger" :loading="killing" @click="confirmOpen = true">
          终止
        </Button>
        <Button v-else size="sm" variant="ghost" @click="toggleCollapse">
          {{ props.live.collapsed ? "展开" : "收起" }}
        </Button>
      </div>
    </header>

    <div class="card-meta mono">
      {{ rec.shellName }} · PID {{ props.live.pid }} · 来源：{{ source }}
    </div>

    <div v-if="running || !props.live.collapsed" class="card-body">
      <LogConsole v-model:paused="props.live.paused" :lines="props.live.logs" />
    </div>

    <footer v-if="result" class="card-result" :class="result.cls">{{ result.text }}</footer>

    <Confirm
      :open="confirmOpen"
      :loading="killing"
      title="终止任务"
      :message="`终止任务「${rec.scriptName}」？将结束整个进程树。`"
      confirm-text="终止"
      @update:open="confirmOpen = $event"
      @confirm="kill"
    />
  </section>
</template>

<style scoped>
.run-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--bg-panel);
}

.card-head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.status-dot {
  flex: none;
  width: 9px;
  height: 9px;
  border-radius: 50%;
}
.status-dot.is-running {
  animation: dot-pulse 1.4s ease-in-out infinite;
}
@keyframes dot-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--status-running) 45%, transparent);
  }
  50% {
    box-shadow: 0 0 0 4px transparent;
  }
}

.card-title {
  flex: 1;
  min-width: 0;
  font-size: var(--font-md);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-running {
  flex: none;
  font-size: var(--font-xs);
  color: var(--status-running);
}
.card-status {
  flex: none;
  font-size: var(--font-xs);
}
.card-dur {
  flex: none;
  font-size: var(--font-xs);
  color: var(--text-tertiary);
}
.card-actions {
  flex: none;
  display: flex;
  gap: var(--space-1);
}

.card-meta {
  font-size: var(--font-xs);
  color: var(--text-secondary);
}

.card-body {
  min-width: 0;
}

.card-result {
  font-size: var(--font-sm);
  font-weight: 500;
}
.card-result.ok {
  color: var(--status-success);
}
.card-result.bad {
  color: var(--status-failed);
}
.card-result.timeout {
  color: var(--status-timeout);
}
.card-result.killed {
  color: var(--status-killed);
}

.mono {
  font-family: var(--font-mono);
}
</style>
