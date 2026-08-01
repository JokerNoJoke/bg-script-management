<script setup lang="ts">
import { computed, onActivated, onMounted, reactive, ref } from "vue";
import { api } from "../api";
import { shellKindLabel, store } from "../store";
import type { ShellConfig, ShellKind } from "../types";
import Button from "./ui/Button.vue";
import Confirm from "./ui/Confirm.vue";
import Input from "./ui/Input.vue";
import Modal from "./ui/Modal.vue";
import Select from "./ui/Select.vue";
import type { SelectOption } from "./ui/Select.vue";
import Table from "./ui/Table.vue";
import type { TableColumn } from "./ui/Table.vue";
import Tag from "./ui/Tag.vue";
import Textarea from "./ui/Textarea.vue";

const kindOptions: SelectOption[] = (
  ["powershell", "cmd", "bash", "sh"] as ShellKind[]
).map((k) => ({ value: k, label: shellKindLabel[k] }));

const columns: TableColumn[] = [
  { key: "name", label: "名称" },
  { key: "kind", label: "种类", slot: "kind" },
  { key: "exe", label: "可执行文件", slot: "exe" },
  { key: "args", label: "参数", slot: "args" },
  { key: "source", label: "来源", slot: "source" },
  { key: "actions", label: "操作", slot: "actions" },
];

const rows = computed(() => store.shells as unknown as Record<string, unknown>[]);

const modalOpen = ref(false);
const editingId = ref<string | null>(null);
const saving = ref(false);
const form = reactive({
  name: "",
  kind: "powershell" as ShellKind,
  exe: "",
  argsText: "",
});
const errors = reactive({ name: false, exe: false });

const deleteTarget = ref<ShellConfig | null>(null);
const deleting = ref(false);

function refreshShells() {
  void store.refreshShells();
}

onMounted(refreshShells);
onActivated(refreshShells);

function openCreate() {
  editingId.value = null;
  form.name = "";
  form.kind = "powershell";
  form.exe = "";
  form.argsText = "";
  errors.name = false;
  errors.exe = false;
  modalOpen.value = true;
}

function openEdit(shell: ShellConfig) {
  editingId.value = shell.id;
  form.name = shell.name;
  form.kind = shell.kind;
  form.exe = shell.exe;
  form.argsText = shell.args.join("\n");
  errors.name = false;
  errors.exe = false;
  modalOpen.value = true;
}

function parseArgs(text: string): string[] {
  return text
    .split(/\r?\n/)
    .flatMap((line) => line.split(","))
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

async function save() {
  errors.name = form.name.trim().length === 0;
  errors.exe = form.exe.trim().length === 0;
  if (errors.name || errors.exe) return;
  saving.value = true;
  const shell: ShellConfig = {
    id: editingId.value ?? "",
    name: form.name.trim(),
    kind: form.kind,
    exe: form.exe.trim(),
    args: parseArgs(form.argsText),
    builtin: false,
  };
  try {
    await api.saveShell(shell);
    store.toast(editingId.value ? "已更新 Shell" : "已新增 Shell", "success");
    modalOpen.value = false;
    await store.refreshShells();
  } catch (e) {
    store.toast(errorMessage(e), "error");
  } finally {
    saving.value = false;
  }
}

function askDelete(shell: ShellConfig) {
  deleteTarget.value = shell;
}

async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target) return;
  deleting.value = true;
  try {
    await api.deleteShell(target.id);
    store.toast(`已删除 Shell「${target.name}」`, "success");
    deleteTarget.value = null;
    await store.refreshShells();
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

function asShell(row: Record<string, unknown>): ShellConfig {
  return row as unknown as ShellConfig;
}

function joinArgs(args: string[]): string {
  return args.length ? args.join(" ") : "—";
}
</script>

<template>
  <div class="shells-view">
    <div class="toolbar">
      <Button size="sm" @click="openCreate">＋ 新增</Button>
    </div>

    <Table :columns="columns" :rows="rows">
      <template #kind="{ row }">
        {{ shellKindLabel[asShell(row).kind] }}
      </template>
      <template #exe="{ value }">
        <code class="mono">{{ value }}</code>
      </template>
      <template #args="{ row }">
        <code class="mono args-cell" :title="asShell(row).args.join(' ')">
          {{ joinArgs(asShell(row).args) }}
        </code>
      </template>
      <template #source="{ row }">
        <Tag v-if="asShell(row).builtin" tone="neutral" label="系统" />
        <span v-else class="custom-source">自定义</span>
      </template>
      <template #actions="{ row }">
        <template v-if="!asShell(row).builtin">
          <button type="button" class="action-btn" @click="openEdit(asShell(row))">编辑</button>
          <button type="button" class="action-btn is-danger" @click="askDelete(asShell(row))">
            删除
          </button>
        </template>
      </template>
    </Table>

    <Modal v-model:open="modalOpen" :title="editingId ? '编辑 Shell' : '新增 Shell'" width="480px">
      <form class="shell-form" @submit.prevent="save">
        <Input v-model="form.name" :error="errors.name" placeholder="显示名称，如 Git Bash">
          <template #label>名称 *</template>
        </Input>
        <div class="form-row">
          <label class="form-label">种类 *</label>
          <Select
            :model-value="form.kind"
            :options="kindOptions"
            @update:model-value="form.kind = $event as ShellKind"
          />
        </div>
        <Input
          v-model="form.exe"
          :error="errors.exe"
          placeholder="可执行文件路径或命令名，如 C:\\Windows\\System32\\bash.exe"
        >
          <template #label>可执行文件 *</template>
        </Input>
        <Textarea
          v-model="form.argsText"
          mono
          placeholder="启动参数：每行一个，或逗号分隔（可为空）"
        >
          <template #label>参数</template>
        </Textarea>
        <div class="form-actions">
          <Button variant="ghost" :disabled="saving" @click="modalOpen = false">取消</Button>
          <Button type="submit" :loading="saving">保存</Button>
        </div>
      </form>
    </Modal>

    <Confirm
      :open="!!deleteTarget"
      :loading="deleting"
      title="删除 Shell"
      :message="`删除 Shell「${deleteTarget?.name ?? ''}」？`"
      confirm-text="删除"
      @update:open="deleteTarget = null"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.shells-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.toolbar {
  display: flex;
  justify-content: flex-end;
}

.mono {
  font-family: var(--font-mono);
  font-size: var(--font-xs);
}

.args-cell {
  display: inline-block;
  max-width: 100%;
  vertical-align: middle;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-source {
  font-size: var(--font-xs);
  color: var(--text-tertiary);
}

.action-btn {
  background: none;
  border: none;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: var(--font-xs);
  color: var(--accent);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}
.action-btn:hover {
  background: var(--bg-hover);
}
.action-btn.is-danger {
  color: var(--danger);
}
.action-btn.is-danger:hover {
  color: var(--danger-hover);
}

.shell-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.form-label {
  font-size: var(--font-xs);
  color: var(--text-secondary);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  margin-top: var(--space-2);
}
</style>
