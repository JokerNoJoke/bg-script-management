<script setup lang="ts">
import { ref, watch } from "vue";
import { statusMeta } from "../store";
import type { LiveRun } from "../store";
import LogConsole from "./LogConsole.vue";
import Modal from "./ui/Modal.vue";

const props = defineProps<{ open: boolean; live: LiveRun | null }>();

const emit = defineEmits<{ (e: "update:open", v: boolean): void }>();

// 暂停状态仅在本次弹窗会话内生效，重新打开时恢复跟随
const paused = ref(false);
watch(
  () => props.open,
  (o) => {
    if (o) paused.value = false;
  },
);
</script>

<template>
  <Modal
    :open="open"
    :title="`运行日志 · ${props.live?.record.scriptName ?? ''}`"
    width="720px"
    @update:open="emit('update:open', $event)"
  >
    <div class="live-viewer">
      <div v-if="props.live" class="live-bar">
        <span class="live-meta mono">
          {{ props.live.record.shellName }} · {{ statusMeta[props.live.record.status].label }} · PID
          {{ props.live.pid }}
        </span>
      </div>
      <LogConsole v-if="props.live" v-model:paused="paused" :lines="props.live.logs" />
    </div>
  </Modal>
</template>

<style scoped>
.live-viewer {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.live-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.live-meta {
  font-size: var(--font-xs);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mono {
  font-family: var(--font-mono);
}
</style>
