<script setup lang="ts">
import { computed } from "vue";
import { store } from "../store";
import Button from "./ui/Button.vue";
import EmptyState from "./ui/EmptyState.vue";
import RunCard from "./RunCard.vue";

// 运行中在前、已结束在后；组内按启动时间倒序
const runs = computed(() => {
  const list = Object.values(store.live);
  return list.sort((a, b) => {
    const ar = a.record.status === "running" ? 0 : 1;
    const br = b.record.status === "running" ? 0 : 1;
    if (ar !== br) return ar - br;
    return b.record.startedAt - a.record.startedAt;
  });
});

const runningCount = computed(
  () => Object.values(store.live).filter((r) => r.record.status === "running").length,
);
const finishedCount = computed(() => runs.value.length - runningCount.value);

function clearFinished() {
  store.clearFinished();
  store.toast("已清除已完成的任务", "info");
}
</script>

<template>
  <div class="console-view">
    <div v-if="runs.length" class="toolbar">
      <span class="toolbar-info">
        {{ runningCount }} 个运行中<template v-if="finishedCount"> · {{ finishedCount }} 个已完成</template>
      </span>
      <Button v-if="finishedCount" size="sm" variant="ghost" @click="clearFinished">
        清除已完成
      </Button>
    </div>

    <EmptyState
      v-if="!runs.length"
      title="暂无运行任务"
      description="从脚本库或「快速执行」启动任务后，将在此实时显示输出日志。"
    />

    <div v-else class="cards">
      <RunCard v-for="r in runs" :key="r.record.id" :live="r" />
    </div>
  </div>
</template>

<style scoped>
.console-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.toolbar-info {
  font-size: var(--font-sm);
  color: var(--text-secondary);
}

.cards {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
</style>
