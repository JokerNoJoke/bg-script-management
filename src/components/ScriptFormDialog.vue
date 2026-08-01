<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { store } from "../store";
import type { ExecType, Script } from "../types";
import { shellOptions } from "./shellOptions";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Modal from "./ui/Modal.vue";
import Select from "./ui/Select.vue";
import Textarea from "./ui/Textarea.vue";

const props = defineProps<{
  open: boolean;
  /** 传入 Script 为编辑模式，null 为新增 */
  script: Script | null;
}>();

const emit = defineEmits<{
  (e: "update:open", v: boolean): void;
  (e: "saved"): void;
}>();

interface EnvRow {
  key: string;
  value: string;
}

const form = reactive({
  name: "",
  description: "",
  shellId: "",
  execType: "command" as ExecType,
  command: "",
  cwd: "",
  env: [] as EnvRow[],
  timeoutSec: "0",
});
const errors = reactive({ name: false, shell: false, command: false });
const saving = ref(false);

function resetForm() {
  form.name = "";
  form.description = "";
  form.shellId = store.defaultShellId;
  form.execType = "command";
  form.command = "";
  form.cwd = "";
  form.env = [];
  form.timeoutSec = "0";
  errors.name = false;
  errors.shell = false;
  errors.command = false;
}

function prefill(script: Script) {
  form.name = script.name;
  form.description = script.description;
  form.shellId = script.shellId;
  form.execType = script.execType;
  form.command = script.command;
  form.cwd = script.cwd ?? "";
  form.env = Object.entries(script.env).map(([key, value]) => ({ key, value }));
  form.timeoutSec = String(script.timeoutSec);
  errors.name = false;
  errors.shell = false;
  errors.command = false;
}

watch(
  () => [props.open, props.script] as const,
  () => {
    if (!props.open) return;
    if (props.script) prefill(props.script);
    else resetForm();
  },
  { immediate: true },
);

async function pickFile() {
  try {
    const picked = await openDialog({ multiple: false, directory: false });
    if (typeof picked === "string") form.command = picked;
  } catch (e) {
    store.toast(errorMessage(e), "error");
  }
}

async function save() {
  errors.name = form.name.trim().length === 0;
  errors.shell = form.shellId.length === 0;
  errors.command = form.command.trim().length === 0;
  if (errors.name || errors.shell || errors.command) return;

  const env: Record<string, string> = {};
  for (const row of form.env) {
    const key = row.key.trim();
    if (key) env[key] = row.value;
  }

  saving.value = true;
  const script: Script = {
    id: props.script?.id ?? "",
    name: form.name.trim(),
    description: form.description.trim(),
    shellId: form.shellId,
    execType: form.execType,
    command: form.command.trim(),
    cwd: form.cwd.trim() || null,
    env,
    timeoutSec: Number(form.timeoutSec) || 0,
    createdAt: props.script?.createdAt ?? 0,
    updatedAt: props.script?.updatedAt ?? 0,
  };
  try {
    await api.saveScript(script);
    store.toast(props.script ? "已更新脚本" : "已新增脚本", "success");
    emit("update:open", false);
    emit("saved");
  } catch (e) {
    store.toast(errorMessage(e), "error");
  } finally {
    saving.value = false;
  }
}

function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

function addEnvRow() {
  form.env.push({ key: "", value: "" });
}
</script>

<template>
  <Modal
    :open="open"
    :title="script ? '编辑脚本' : '新增脚本'"
    width="560px"
    @update:open="emit('update:open', $event)"
  >
    <form class="script-form" @submit.prevent="save">
      <p v-if="script" class="script-id">ID：{{ script.id }}</p>
      <div class="grid">
        <Input v-model="form.name" :error="errors.name" placeholder="如：部署到测试环境">
          <template #label>名称 *</template>
        </Input>
        <div class="field">
          <label class="field-label">Shell *</label>
          <Select
            :model-value="form.shellId"
            :options="shellOptions()"
            placeholder="请选择 Shell"
            @update:model-value="form.shellId = $event"
          />
          <span v-if="errors.shell" class="hint is-error">请选择 Shell</span>
        </div>
        <Input v-model="form.description" placeholder="可选，说明用途">
          <template #label>说明</template>
        </Input>
        <div class="field">
          <label class="field-label">执行类型</label>
          <div class="seg">
            <button
              type="button"
              class="seg-btn"
              :class="{ 'is-active': form.execType === 'command' }"
              @click="form.execType = 'command'"
            >
              命令
            </button>
            <button
              type="button"
              class="seg-btn"
              :class="{ 'is-active': form.execType === 'file' }"
              @click="form.execType = 'file'"
            >
              文件
            </button>
          </div>
        </div>
      </div>

      <div class="field full">
        <label class="field-label">{{ form.execType === "command" ? "命令 *" : "文件路径 *" }}</label>
        <Textarea
          v-if="form.execType === 'command'"
          v-model="form.command"
          mono
          placeholder="要执行的命令内容，如 Write-Host hi"
          :error="errors.command"
        />
        <div v-else class="file-row">
          <Input v-model="form.command" mono :error="errors.command" placeholder="脚本文件绝对路径" />
          <Button variant="secondary" type="button" @click="pickFile">选择文件</Button>
        </div>
        <span v-if="errors.command" class="hint is-error">请输入命令内容或文件路径</span>
      </div>

      <div class="field full">
        <Input v-model="form.cwd" mono placeholder="可选，默认沿用进程当前目录">
          <template #label>工作目录</template>
        </Input>
      </div>

      <div class="grid">
        <Input v-model="form.timeoutSec" type="number" min="0" placeholder="0 = 不限时">
          <template #label>超时秒数</template>
        </Input>
      </div>

      <template v-if="form.env.length">
        <div class="field full">
          <label class="field-label">环境变量</label>
          <div v-for="(row, i) in form.env" :key="i" class="env-row">
            <Input v-model="row.key" mono placeholder="变量名，如 FOO" />
            <span class="env-eq">=</span>
            <Input v-model="row.value" mono placeholder="值，可为空" />
            <button
              type="button"
              class="env-del"
              aria-label="删除该变量"
              @click="form.env.splice(i, 1)"
            >
              ✕
            </button>
          </div>
        </div>
      </template>
      <div class="full">
        <Button variant="ghost" size="sm" type="button" @click="addEnvRow">＋ 添加环境变量</Button>
      </div>

      <div class="form-actions full">
        <Button variant="ghost" :disabled="saving" type="button" @click="emit('update:open', false)">
          取消
        </Button>
        <Button type="submit" :loading="saving">保存</Button>
      </div>
    </form>
  </Modal>
</template>

<style scoped>
.script-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.script-id {
  font-family: var(--font-mono);
  font-size: var(--font-xs);
  color: var(--text-tertiary);
  word-break: break-all;
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
}

.full {
  grid-column: 1 / -1;
}

.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.field-label {
  font-size: var(--font-xs);
  color: var(--text-secondary);
}

.hint {
  font-size: var(--font-xs);
  color: var(--text-tertiary);
}
.hint.is-error {
  color: var(--danger);
}

.seg {
  display: flex;
  gap: 2px;
  height: 32px;
  padding: 2px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-page);
}
.seg-btn {
  flex: 1;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: var(--font-sm);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}
.seg-btn.is-active {
  background: var(--bg-panel);
  color: var(--accent);
  font-weight: 500;
  box-shadow: var(--shadow-sm);
}

.file-row {
  display: flex;
  gap: var(--space-2);
}
.file-row .field {
  flex: 1;
}

.env-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.env-row .field {
  flex: 1;
}
.env-row .field:first-child {
  flex: 0 0 40%;
}
.env-eq {
  flex: none;
  color: var(--text-tertiary);
}
.env-del {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  font-size: var(--font-xs);
  cursor: pointer;
}
.env-del:hover {
  background: var(--bg-hover);
  color: var(--danger);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  margin-top: var(--space-2);
  padding-top: var(--space-3);
  border-top: 1px solid var(--border);
}
</style>
