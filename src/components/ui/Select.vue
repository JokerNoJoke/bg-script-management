<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: SelectOption[];
    placeholder?: string;
    disabled?: boolean;
  }>(),
  { placeholder: "请选择", disabled: false },
);

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

const open = ref(false);
const highlight = ref(-1);
const rootEl = ref<HTMLDivElement | null>(null);

const selectedLabel = computed(
  () => props.options.find((o) => o.value === props.modelValue)?.label ?? props.placeholder,
);

function toggle() {
  if (props.disabled) return;
  open.value = !open.value;
  if (open.value) {
    highlight.value = Math.max(0, props.options.findIndex((o) => o.value === props.modelValue));
  }
}

function choose(o: SelectOption) {
  if (o.disabled) return;
  emit("update:modelValue", o.value);
  close();
}

function close() {
  open.value = false;
  highlight.value = -1;
}

function onDocPointer(e: PointerEvent) {
  if (!open.value) return;
  if (!rootEl.value?.contains(e.target as Node)) close();
}

function onDocKeydown(e: KeyboardEvent) {
  if (!open.value) return;
  const list = props.options;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    highlight.value = (highlight.value + 1) % Math.max(1, list.length);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    highlight.value = (highlight.value - 1 + Math.max(1, list.length)) % Math.max(1, list.length);
  } else if (e.key === "Enter") {
    e.preventDefault();
    const o = list[highlight.value];
    if (o && !o.disabled) {
      emit("update:modelValue", o.value);
      close();
    }
  } else if (e.key === "Escape") {
    close();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointer);
  document.addEventListener("keydown", onDocKeydown);
});
onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onDocPointer);
  document.removeEventListener("keydown", onDocKeydown);
});
</script>

<template>
  <div ref="rootEl" class="select">
    <button
      class="select-trigger"
      :class="{ 'is-open': open }"
      type="button"
      :disabled="disabled"
      aria-haspopup="listbox"
      :aria-expanded="open"
      @click="toggle"
    >
      <span class="select-value" :class="{ 'is-placeholder': !modelValue }">{{ selectedLabel }}</span>
      <svg class="select-caret" viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
        <path d="m2.5 4.5 3.5 3.5 3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <Transition name="select">
      <ul v-if="open" class="select-list" role="listbox">
        <li
          v-for="(o, i) in options"
          :key="o.value"
          role="option"
          :aria-selected="o.value === modelValue"
          class="select-option"
          :class="{
            'is-selected': o.value === modelValue,
            'is-highlight': i === highlight,
            'is-disabled': o.disabled,
          }"
          @click="choose(o)"
          @pointerenter="highlight = i"
        >
          {{ o.label }}
        </li>
        <li v-if="options.length === 0" class="select-empty">暂无选项</li>
      </ul>
    </Transition>
  </div>
</template>

<style scoped>
.select {
  position: relative;
  min-width: 0;
}
.select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-primary);
  font-size: var(--font-sm);
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s ease;
}
.select-trigger:hover,
.select-trigger.is-open {
  border-color: var(--border-strong);
}
.select-trigger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.select-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.select-value.is-placeholder {
  color: var(--text-tertiary);
}
.select-caret {
  flex: none;
  color: var(--text-tertiary);
}
.select-list {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 50;
  max-height: 240px;
  overflow-y: auto;
  margin: 0;
  padding: 4px;
  list-style: none;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  box-shadow: var(--shadow-sm);
}
.select-option {
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  font-size: var(--font-sm);
  color: var(--text-primary);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.select-option:hover,
.select-option.is-highlight {
  background: var(--bg-hover);
}
.select-option.is-selected {
  color: var(--accent);
  font-weight: 500;
}
.select-option.is-disabled {
  color: var(--text-tertiary);
  cursor: not-allowed;
}
.select-empty {
  padding: 8px 10px;
  font-size: var(--font-xs);
  color: var(--text-tertiary);
  text-align: center;
}
.select-enter-active,
.select-leave-active {
  transition: opacity 0.12s ease, transform 0.12s ease;
}
.select-enter-from,
.select-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
