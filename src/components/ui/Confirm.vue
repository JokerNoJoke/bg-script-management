<script setup lang="ts">
import Button from "./Button.vue";
import Modal from "./Modal.vue";

withDefaults(
  defineProps<{
    open: boolean;
    title?: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    loading?: boolean;
  }>(),
  { title: "确认操作", confirmText: "确认", cancelText: "取消", loading: false },
);

const emit = defineEmits<{
  (e: "update:open", v: boolean): void;
  (e: "confirm"): void;
}>();
</script>

<template>
  <Modal :open="open" :title="title" width="400px" @update:open="emit('update:open', $event)">
    <p class="confirm-message">{{ message }}</p>
    <template #footer>
      <Button variant="ghost" :disabled="loading" @click="emit('update:open', false)">
        {{ cancelText }}
      </Button>
      <Button variant="danger" :loading="loading" @click="emit('confirm')">
        {{ confirmText }}
      </Button>
    </template>
  </Modal>
</template>

<style scoped>
.confirm-message {
  font-size: var(--font-sm);
  color: var(--text-primary);
  line-height: 1.6;
  word-break: break-word;
}
</style>
