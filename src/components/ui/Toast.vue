<script setup lang="ts">
import { store } from "../../store";

function iconFor(type: string): string {
  if (type === "success") return "✓";
  if (type === "error") return "✕";
  return "i";
}
</script>

<template>
  <Teleport to="body">
    <div class="toast-wrap">
      <TransitionGroup name="toast">
        <div v-for="t in store.toasts" :key="t.id" class="toast" :class="`toast-${t.type}`">
          <span class="toast-icon">{{ iconFor(t.type) }}</span>
          <span class="toast-msg">{{ t.msg }}</span>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-wrap {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 200;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  pointer-events: none;
}
.toast {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 420px;
  padding: 8px 14px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  box-shadow: var(--shadow-sm);
  font-size: var(--font-sm);
  color: var(--text-primary);
}
.toast-icon {
  flex: none;
  font-size: var(--font-sm);
  font-weight: 600;
}
.toast-success .toast-icon {
  color: var(--status-success);
}
.toast-error .toast-icon {
  color: var(--status-failed);
}
.toast-info .toast-icon {
  color: var(--text-secondary);
}
.toast-msg {
  overflow-wrap: anywhere;
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
