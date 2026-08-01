<script setup lang="ts">
import Spinner from "./Spinner.vue";

withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "danger" | "ghost" | "text";
    size?: "sm" | "md";
    loading?: boolean;
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  {
    variant: "primary",
    size: "md",
    loading: false,
    disabled: false,
    type: "button",
  },
);
</script>

<template>
  <button
    class="btn"
    :class="[`btn-${variant}`, `btn-${size}`]"
    :type="type"
    :disabled="disabled || loading"
  >
    <Spinner v-if="loading" class="btn-spinner" />
    <span class="btn-content"><slot /></span>
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  cursor: pointer;
  font-weight: 500;
  white-space: nowrap;
  transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
}
.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.btn-sm {
  height: 28px;
  padding: 0 10px;
  font-size: var(--font-xs);
}
.btn-md {
  height: 34px;
  padding: 0 14px;
  font-size: var(--font-sm);
}
.btn-content {
  display: inline-flex;
  align-items: center;
}
.btn-spinner {
  margin-right: 4px;
}
.btn-primary {
  background: var(--accent);
  color: var(--accent-contrast);
}
.btn-primary:not(:disabled):hover {
  background: var(--accent-hover);
}
.btn-secondary {
  background: var(--bg-panel);
  border-color: var(--border-strong);
  color: var(--text-primary);
}
.btn-secondary:not(:disabled):hover {
  background: var(--bg-hover);
}
.btn-danger {
  background: var(--danger);
  color: #ffffff;
}
.btn-danger:not(:disabled):hover {
  background: var(--danger-hover);
}
.btn-ghost {
  background: var(--bg-panel);
  border-color: var(--border);
  color: var(--text-primary);
}
.btn-ghost:not(:disabled):hover {
  background: var(--bg-hover);
  border-color: var(--border-strong);
}
.btn-text {
  background: transparent;
  color: var(--accent);
}
.btn-text:not(:disabled):hover {
  background: var(--bg-hover);
}
</style>
