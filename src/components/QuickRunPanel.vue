<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { store } from "../store";
import { shellOptions } from "./shellOptions";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Modal from "./ui/Modal.vue";
import Select from "./ui/Select.vue";
import Textarea from "./ui/Textarea.vue";

const props = defineProps<{ open: boolean }>();

const emit = defineEmits<{ (e: "update:open", v: boolean): void }>();

const form = reactive({
  shellId: "",
  cwd: "",
  command: "",
});
const errors = reactive({ shell: false, command: false });
const running = ref(false);

watch(
  () => props.open,
  (open) => {
    if (!open) return;
    form.shellId = store.defaultShellId;
    form.cwd = "";
    form.command = "";
    errors.shell = false;
    errors.command = false;
  },
  { immediate: true },
);

async function run() {
  errors.shell = form.shellId.length === 0;
  errors.command = form.command.trim().length === 0;
  if (errors.shell || errors.command) return;

  running.value = true;
  const record = await store.startRun({
    scriptId: null,
    scriptName: "快速执行",
    shellId: form.shellId,
    command: form.command.trim(),
    execType: "command",
    cwd: form.cwd.trim() || null,
    env: {},
    timeoutSec: 0,
  });
  running.value = false;
  if (record) {
    emit("update:open", false);
    store.toast("已启动：快速执行", "success");
  }
}
</script>

<template>
  <Modal :open="open" title="快速执行" width="560px" @update:open="emit('update:open', $event)">
    <div class="quick-form">
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

      <Input v-model="form.cwd" mono placeholder="可选，默认沿用进程当前目录">
        <template #label>工作目录</template>
      </Input>

      <div class="field">
        <label class="field-label">命令 / 脚本内容 *</label>
        <Textarea
          v-model="form.command"
          mono
          :min-height="'160px'"
          :error="errors.command"
          placeholder="粘贴要执行的命令或脚本内容，不会保存进脚本库"
        />
        <span v-if="errors.command" class="hint is-error">请输入命令内容</span>
      </div>

      <div class="form-actions">
        <Button variant="ghost" :disabled="running" type="button" @click="emit('update:open', false)">
          取消
        </Button>
        <Button :loading="running" @click="run">运行</Button>
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.quick-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
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

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
}
</style>
