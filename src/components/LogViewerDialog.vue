<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import { api } from "../api";
import type { RunRecord } from "../types";
import Button from "./ui/Button.vue";
import EmptyState from "./ui/EmptyState.vue";
import Modal from "./ui/Modal.vue";
import Spinner from "./ui/Spinner.vue";

const props = defineProps<{ open: boolean; run: RunRecord | null }>();

const emit = defineEmits<{ (e: "update:open", v: boolean): void }>();

// 运行中每轮询一次磁盘日志，近实时跟随；已结束只读一次。
const POLL_MS = 1500;

const text = ref("");
const loading = ref(false);
const failed = ref(false);

const logEl = ref<HTMLPreElement | null>(null);
let stick = true;
function onScroll() {
  const el = logEl.value;
  if (!el) return;
  stick = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
}

let timer: number | undefined;
let seq = 0;
let lastRunId: string | undefined;

async function load() {
  const run = props.run;
  if (!run) return;
  const mySeq = ++seq;
  loading.value = text.value === "";
  failed.value = false;
  try {
    const t = await api.getRunLog(run.id);
    if (mySeq !== seq) return;
    text.value = t;
    void nextTick(() => {
      if (stick && logEl.value) logEl.value.scrollTop = logEl.value.scrollHeight;
    });
  } catch {
    if (mySeq !== seq) return;
    if (run.status === "running") {
      // 运行中日志文件可能尚未创建（尚无输出），视为空内容
      text.value = "";
    } else {
      failed.value = true;
    }
  } finally {
    if (mySeq === seq) loading.value = false;
  }
}

function stopPolling() {
  window.clearInterval(timer);
  timer = undefined;
}

watch(
  () => [props.open, props.run, props.run?.status],
  () => {
    stopPolling();
    const run = props.open ? props.run : null;
    if (!run) return;
    if (run.id !== lastRunId) {
      lastRunId = run.id;
      text.value = "";
      failed.value = false;
      stick = true;
    }
    void load();
    if (run.status === "running") {
      timer = window.setInterval(load, POLL_MS);
    }
  },
  { immediate: true },
);

onBeforeUnmount(stopPolling);

async function copy() {
  try {
    await navigator.clipboard.writeText(text.value);
  } catch {
    /* 剪贴板不可用时忽略 */
  }
}
</script>

<template>
  <Modal
    :open="open"
    :title="`运行日志 · ${props.run?.scriptName ?? ''}`"
    width="720px"
    @update:open="emit('update:open', $event)"
  >
    <div class="viewer">
      <div class="viewer-bar">
        <span v-if="props.run" class="viewer-meta mono">
          {{ props.run.shellName }} · {{ new Date(props.run.startedAt).toLocaleString() }}
        </span>
        <Button
          size="sm"
          variant="ghost"
          :disabled="loading || failed || !text"
          @click="copy"
        >
          复制全部
        </Button>
      </div>

      <div class="viewer-body">
        <div v-if="loading" class="viewer-state">
          <Spinner :size="20" />
        </div>
        <div v-else-if="failed && !text" class="viewer-state">
          <EmptyState title="日志不可用" description="日志文件不存在或读取失败。" />
        </div>
        <pre v-else-if="text" ref="logEl" class="viewer-log" @scroll.passive="onScroll">{{ text }}</pre>
        <div v-else class="viewer-state">
          <EmptyState title="暂无日志内容" description="该次运行没有产生任何输出。" />
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.viewer {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.viewer-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.viewer-meta {
  font-size: var(--font-xs);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.viewer-body {
  min-height: 160px;
}
.viewer-state {
  display: flex;
  align-items: center;
  justify-content: center;
}

.viewer-log {
  margin: 0;
  max-height: 60vh;
  overflow: auto;
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--bg-log);
  color: var(--log-text);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.mono {
  font-family: var(--font-mono);
}
</style>
