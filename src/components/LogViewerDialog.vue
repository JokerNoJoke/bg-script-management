<script setup lang="ts">
import { ref, watch } from "vue";
import { api } from "../api";
import type { RunRecord } from "../types";
import Button from "./ui/Button.vue";
import EmptyState from "./ui/EmptyState.vue";
import Modal from "./ui/Modal.vue";
import Spinner from "./ui/Spinner.vue";

const props = defineProps<{ open: boolean; run: RunRecord | null }>();

const emit = defineEmits<{ (e: "update:open", v: boolean): void }>();

const text = ref("");
const loading = ref(false);
const failed = ref(false);

watch(
  () => props.open,
  async (open) => {
    if (!open || !props.run) return;
    text.value = "";
    failed.value = false;
    loading.value = true;
    try {
      text.value = await api.getRunLog(props.run.id);
    } catch {
      failed.value = true;
    } finally {
      loading.value = false;
    }
  },
);

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
        <div v-else-if="failed" class="viewer-state">
          <EmptyState title="日志不可用" description="日志文件不存在或读取失败。" />
        </div>
        <pre v-else-if="text" class="viewer-log">{{ text }}</pre>
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
