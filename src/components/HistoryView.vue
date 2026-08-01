<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { api } from "../api";
import { statusMeta, store } from "../store";
import type { RunRecord, RunStatus } from "../types";
import Button from "./ui/Button.vue";
import Confirm from "./ui/Confirm.vue";
import EmptyState from "./ui/EmptyState.vue";
import Input from "./ui/Input.vue";
import Select from "./ui/Select.vue";
import type { SelectOption } from "./ui/Select.vue";
import Table from "./ui/Table.vue";
import type { TableColumn } from "./ui/Table.vue";
import Tag from "./ui/Tag.vue";
import LogViewerDialog from "./LogViewerDialog.vue";

const HISTORY_LIMIT = 500;

function refresh() {
  void store.refreshHistory();
}
onMounted(refresh);
onActivated(refresh);

// 筛选
const statusFilter = ref("");
const scriptFilter = ref("");
const search = ref("");
let searchTimer: number | undefined;
function onSearch(v: string) {
  window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    search.value = v;
  }, 200);
}

const statusOptions = computed<SelectOption[]>(() => [
  { value: "", label: "全部状态" },
  ...(Object.keys(statusMeta) as RunStatus[])
    .filter((s) => s !== "running")
    .map((s) => ({ value: s, label: statusMeta[s].label })),
]);

const scriptOptions = computed<SelectOption[]>(() => {
  const names = new Set<string>();
  for (const r of store.runs) names.add(r.scriptName || "快速执行");
  const list = [...names].sort((a, b) => a.localeCompare(b, "zh"));
  return [{ value: "", label: "全部脚本" }, ...list.map((n) => ({ value: n, label: n }))];
});

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  return store.runs.filter((r) => {
    if (statusFilter.value && r.status !== statusFilter.value) return false;
    if (scriptFilter.value && (r.scriptName || "快速执行") !== scriptFilter.value) return false;
    if (q) {
      const hay = `${r.scriptName} ${r.command} ${r.shellName}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
});

const rows = computed(() => filtered.value as unknown as Record<string, unknown>[]);

const nearLimit = computed(() => store.runs.length >= HISTORY_LIMIT - 50);

const columns: TableColumn[] = [
  { key: "startedAt", label: "开始时间", width: "150px", slot: "startedAt" },
  { key: "script", label: "脚本", slot: "script" },
  { key: "shell", label: "Shell", width: "110px", slot: "shell" },
  { key: "status", label: "状态", width: "90px", slot: "status" },
  { key: "exitCode", label: "退出码", width: "64px", align: "center", slot: "exitCode" },
  { key: "duration", label: "耗时", width: "80px", slot: "duration" },
  { key: "actions", label: "操作", width: "110px", slot: "actions" },
];

const pad = (n: number) => String(n).padStart(2, "0");
function fmtStart(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function fmtDur(r: RunRecord): string {
  if (r.finishedAt == null) return "—";
  const sec = Math.max(0, (r.finishedAt - r.startedAt) / 1000);
  if (sec < 60) return `${sec.toFixed(1)}s`;
  const m = Math.floor(sec / 60);
  const s = Math.round(sec % 60);
  if (m < 60) return `${m}m ${s}s`;
  const h = Math.floor(m / 60);
  return `${h}h ${m % 60}m`;
}

// 日志回看
const logOpen = ref(false);
const logRun = ref<RunRecord | null>(null);
function openLog(r: RunRecord) {
  logRun.value = r;
  logOpen.value = true;
}

// 重跑：用记录重建 RunInput
async function rerun(r: RunRecord) {
  const script = store.scripts.find((s) => s.id === r.scriptId);
  const record = await store.startRun({
    scriptId: r.scriptId,
    scriptName: r.scriptName,
    shellId: r.shellId,
    command: r.command,
    execType: script?.execType ?? "command",
    cwd: r.cwd,
    env: script?.env ?? {},
    timeoutSec: script?.timeoutSec ?? 0,
  });
  if (record) store.toast(`已重新启动：${r.scriptName}`, "success");
}

// 清空：无脚本筛选清全部；有对应脚本筛选只清该脚本
const clearTarget = computed<{ message: string; scriptId: string | null }>(() => {
  const f = scriptFilter.value;
  if (f) {
    const s = store.scripts.find((x) => x.name === f);
    if (s) return { message: `清空「${f}」的历史？此操作不可撤销。`, scriptId: s.id };
  }
  return { message: "清空全部运行历史？此操作不可撤销。", scriptId: null };
});

const clearOpen = ref(false);
const clearing = ref(false);

async function confirmClear() {
  clearing.value = true;
  try {
    await api.clearHistory(clearTarget.value.scriptId ?? undefined);
    store.toast(
      clearTarget.value.scriptId ? `已清空「${scriptFilter.value}」的历史` : "已清空全部运行历史",
      "success",
    );
    clearOpen.value = false;
    await store.refreshHistory();
  } catch (e) {
    store.toast(errorMessage(e), "error");
  } finally {
    clearing.value = false;
  }
}

function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

function asRun(row: Record<string, unknown>): RunRecord {
  return row as unknown as RunRecord;
}
</script>

<template>
  <div class="history-view">
    <div v-if="nearLimit" class="limit-banner">
      历史已达上限（{{ HISTORY_LIMIT }} 条），将自动淘汰最旧记录。
    </div>

    <template v-if="store.runs.length">
      <div class="toolbar">
        <Select
          class="filter"
          :model-value="statusFilter"
          :options="statusOptions"
          @update:model-value="statusFilter = $event"
        />
        <Select
          class="filter"
          :model-value="scriptFilter"
          :options="scriptOptions"
          @update:model-value="scriptFilter = $event"
        />
        <Input
          class="search"
          :model-value="search"
          placeholder="搜索脚本 / 命令 / Shell"
          aria-label="搜索历史"
          @update:model-value="onSearch"
        />
        <div class="toolbar-actions">
          <Button variant="ghost" @click="clearOpen = true">清空</Button>
        </div>
      </div>

      <Table :columns="columns" :rows="rows">
        <template #startedAt="{ row }">
          <span class="mono time-cell">{{ fmtStart(asRun(row).startedAt) }}</span>
        </template>
        <template #script="{ row }">
          <span class="script-cell" :class="{ 'is-quick': !asRun(row).scriptId }">
            {{ asRun(row).scriptName }}
          </span>
        </template>
        <template #shell="{ row }">
          <span class="mono">{{ asRun(row).shellName }}</span>
        </template>
        <template #status="{ row }">
          <Tag :tone="statusMeta[asRun(row).status].tone">{{ statusMeta[asRun(row).status].label }}</Tag>
        </template>
        <template #exitCode="{ row }">
          <span class="mono">{{ asRun(row).exitCode ?? "—" }}</span>
        </template>
        <template #duration="{ row }">
          <span class="mono">{{ fmtDur(asRun(row)) }}</span>
        </template>
        <template #actions="{ row }">
          <span class="row-actions">
            <Button size="sm" variant="ghost" @click="openLog(asRun(row))">日志</Button>
            <Button size="sm" variant="secondary" @click="rerun(asRun(row))">重跑</Button>
          </span>
        </template>
        <template #empty>
          <span>未找到匹配的记录</span>
        </template>
      </Table>
    </template>

    <EmptyState
      v-else
      title="暂无运行记录"
      description="运行脚本或快速执行后，每次运行的记录与日志都会保存在这里。"
    />

    <LogViewerDialog v-model:open="logOpen" :run="logRun" />

    <Confirm
      :open="clearOpen"
      :loading="clearing"
      title="清空历史"
      :message="clearTarget.message"
      confirm-text="清空"
      @update:open="clearOpen = $event"
      @confirm="confirmClear"
    />
  </div>
</template>

<style scoped>
.history-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.limit-banner {
  padding: 8px 12px;
  border: 1px solid color-mix(in srgb, var(--status-timeout) 40%, transparent);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--status-timeout) 10%, transparent);
  color: var(--status-timeout);
  font-size: var(--font-xs);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}
.filter {
  width: 140px;
}
.search {
  flex: 1;
  min-width: 180px;
  max-width: 320px;
}
.toolbar-actions {
  margin-left: auto;
}

.mono {
  font-family: var(--font-mono);
  font-size: var(--font-xs);
}
.time-cell {
  color: var(--text-secondary);
}
.script-cell {
  color: var(--text-primary);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.script-cell.is-quick {
  color: var(--text-tertiary);
  font-weight: 400;
}

.row-actions {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
