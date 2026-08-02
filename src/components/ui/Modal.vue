<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";

const props = withDefaults(
  defineProps<{ open: boolean; title?: string; width?: string }>(),
  { title: "", width: "480px" },
);

const emit = defineEmits<{ (e: "update:open", v: boolean): void }>();

function close() {
  emit("update:open", false);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) close();
}

onMounted(() => document.addEventListener("keydown", onKey));
onBeforeUnmount(() => document.removeEventListener("keydown", onKey));
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="open" class="modal-mask" @click.self="close">
        <div class="modal-card" :style="{ width }" role="dialog" aria-modal="true" :aria-label="title">
          <header class="modal-head">
            <h3 class="modal-title">{{ title }}</h3>
            <button class="modal-close" type="button" aria-label="关闭" @click="close">
              <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                <path d="m4 4 8 8M12 4l-8 8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
            </button>
          </header>
          <div class="modal-body">
            <slot />
          </div>
          <footer v-if="$slots.footer" class="modal-foot">
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-6);
  background: rgba(10, 13, 17, 0.5);
}
.modal-card {
  display: flex;
  flex-direction: column;
  max-width: 100%;
  max-height: 80vh;
  border-radius: var(--radius-lg);
  background: var(--bg-panel);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-lg);
  color: var(--text-primary);
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border);
  flex: none;
}
.modal-title {
  font-size: var(--font-lg);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.modal-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}
.modal-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.modal-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  padding: var(--space-5);
  overflow-y: auto;
}
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--border);
  flex: none;
}
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
