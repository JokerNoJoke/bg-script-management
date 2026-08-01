<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onMounted, ref } from "vue";
import { api } from "../api";
import { shellLabel, store } from "../store";
import type { ExecType, Script } from "../types";
import Button from "./ui/Button.vue";
import Confirm from "./ui/Confirm.vue";
import EmptyState from "./ui/EmptyState.vue";
import Input from "./ui/Input.vue";
import Table from "./ui/Table.vue";
import type { TableColumn } from "./ui/Table.vue";
import Tag from "./ui/Tag.vue";
import QuickRunPanel from "./QuickRunPanel.vue";
import ScriptFormDialog from "./ScriptFormDialog.vue";

const execTypeLabel: Record<ExecType, string> = { command: "命令", file: "文件" };

const columns: TableColumn[] = [
  { key: "name", label: "名称", slot: "name" },
  { key: "shell", label: "Shell", slot: "shell" },
  { key: "type", label: "类型", slot: "type" },
  { key: "command", label: "命令 / 路径", slot: "command" },
  { key: "lastRun", label: "最后运行", slot: "lastRun" },
  { key: "actions", label: "操作", slot: "actions" },
];

const search = ref("");
let searchTimer: number | undefined;
function onSearch(value: string) {
  window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    search.value = value;
  }, 200);
}

function refresh() {
  void store.refreshScripts();
  void store.refreshHistory();
}

onMounted(refresh);
onActivated(refresh);

// 每 30s 刷新一次「最后运行」相对时间
const nowTick = ref(0);
let tickTimer: number | undefined;
onMounted(() => {
  tickTimer = window.setInterval(() => {
    nowTick.value = Date.now();
  }, 30000);
});
onBeforeUnmount(() => window.clearInterval(tickTimer));

const filteredScripts = computed(() => {
  void nowTick.value;
  const q = search.value.trim().toLowerCase();
  if (!q) return store.scripts;
  return store.scripts.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.command.toLowerCase().includes(q) ||
      (s.cwd ?? "").toLowerCase().includes(q),
  );
});

const rows = computed(() => filteredScripts.value as unknown as Record<string, unknown>[]);

const lastRunMap = computed<Record<string, string>>(() => {
  void nowTick.value;
  const map: Record<string, string> = {};
  for (const s of store.scripts) map[s.id] = lastRunLabel(s.id);
  return map;
});

function lastRunLabel(scriptId: string): string {
  const rec = store.runs.find((r) => r.scriptId === scriptId);
  return rec ? relativeTime(rec.startedAt) : "—";
}

function relativeTime(ms: number): string {
  const diff = Math.max(0, Date.now() - ms);
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return "刚刚";
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} 分钟前`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} 小时前`;
  const day = Math.floor(hr / 24);
  if (day < 30) return `${day} 天前`;
  const mon = Math.floor(day / 30);
  if (mon < 12) return `${mon} 个月前`;
  return `${Math.floor(mon / 12)} 年前`;
}

// 弹窗状态
const quickOpen = ref(false);
const formOpen = ref(false);
const editing = ref<Script | null>(null);

function openCreate() {
  editing.value = null;
  formOpen.value = true;
}

function openEdit(s: Script) {
  editing.value = s;
  formOpen.value = true;
}

// 运行
const runningIds = ref<Set<string>>(new Set());
async function run(s: Script) {
  if (runningIds.value.has(s.id)) return;
  runningIds.value.add(s.id);
  const record = await store.startRun({
    scriptId: s.id,
    scriptName: s.name,
    shellId: s.shellId,
    command: s.command,
    execType: s.execType,
    cwd: s.cwd,
    env: s.env,
    timeoutSec: s.timeoutSec,
  });
  runningIds.value.delete(s.id);
  if (record) store.toast(`已启动：${s.name}`, "success");
}

// 删除
const deleteTarget = ref<Script | null>(null);
const deleting = ref(false);

async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target) return;
  deleting.value = true;
  try {
    await api.deleteScript(target.id);
    store.toast(`已删除脚本「${target.name}」`, "success");
    deleteTarget.value = null;
    await store.refreshScripts();
  } catch (e) {
    store.toast(errorMessage(e), "error");
  } finally {
    deleting.value = false;
  }
}

function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

function asScript(row: Record<string, unknown>): Script {
  return row as unknown as Script;
}
</script>

<template>
  <div class="scripts-view">
    <div v-if="store.scripts.length" class="toolbar">
      <Input
        class="search"
        :model-value="search"
        placeholder="搜索名称 / 命令 / 路径"
        aria-label="搜索脚本"
        @update:model-value="onSearch"
      />
      <div class="toolbar-actions">
        <Button variant="ghost" @click="quickOpen = true">快速执行</Button>
        <Button @click="openCreate">＋ 新增脚本</Button>
      </div>
    </div>

    <EmptyState
      v-if="!store.scripts.length"
      title="还没有脚本"
      description="点击右上角「新增脚本」，把常用命令固化成可复用脚本。"
    >
      <Button variant="secondary" @click="quickOpen = true">快速执行</Button>
      <Button @click="openCreate">＋ 新增脚本</Button>
    </EmptyState>

    <Table v-else :columns="columns" :rows="rows">
      <template #name="{ row }">
        <span class="name-cell">{{ asScript(row).name }}</span>
      </template>
      <template #shell="{ row }">
        <Tag tone="neutral">{{ shellLabel(asScript(row).shellId) }}</Tag>
      </template>
      <template #type="{ row }">
        {{ execTypeLabel[asScript(row).execType] }}
      </template>
      <template #command="{ row }">
        <code class="mono cmd-cell" :title="asScript(row).command">{{ asScript(row).command }}</code>
      </template>
      <template #lastRun="{ row }">
        <span class="last-run">{{ lastRunMap[asScript(row).id] }}</span>
      </template>
      <template #actions="{ row }">
        <span class="row-actions">
          <Button
            size="sm"
            variant="secondary"
            :loading="runningIds.has(asScript(row).id)"
            @click="run(asScript(row))"
          >
            运行
          </Button>
          <Button size="sm" variant="ghost" @click="openEdit(asScript(row))">编辑</Button>
          <Button size="sm" variant="text" class="del-btn" @click="deleteTarget = asScript(row)">
            删除
          </Button>
        </span>
      </template>
      <template #empty>
        <span>未找到匹配「{{ search }}」的脚本</span>
      </template>
    </Table>

    <ScriptFormDialog
      v-model:open="formOpen"
      :script="editing"
      @saved="refresh"
    />
    <QuickRunPanel v-model:open="quickOpen" />

    <Confirm
      :open="!!deleteTarget"
      :loading="deleting"
      title="删除脚本"
      :message="`删除脚本「${deleteTarget?.name ?? ''}」？此操作不可撤销。`"
      confirm-text="删除"
      @update:open="deleteTarget = null"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.scripts-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  justify-content: space-between;
}
.search {
  max-width: 320px;
}
.toolbar-actions {
  display: flex;
  gap: var(--space-2);
}

.name-cell {
  color: var(--accent);
  font-weight: 600;
}

.mono {
  font-family: var(--font-mono);
  font-size: var(--font-xs);
}

.cmd-cell {
  display: inline-block;
  max-width: 100%;
  vertical-align: middle;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.last-run {
  font-size: var(--font-xs);
  color: var(--text-tertiary);
}

.row-actions {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}
.del-btn {
  color: var(--danger);
}
.del-btn:hover {
  color: var(--danger-hover);
}
</style>
