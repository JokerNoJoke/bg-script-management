<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    error?: boolean;
    disabled?: boolean;
    type?: string;
  }>(),
  { placeholder: "", error: false, disabled: false, type: "text" },
);

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();
</script>

<template>
  <div class="field">
    <label v-if="$slots.label" class="field-label"><slot name="label" /></label>
    <input
      class="input"
      :class="{ 'is-error': error }"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </div>
</template>

<style scoped>
.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.field-label {
  font-size: var(--font-xs);
  color: var(--text-secondary);
}
.input {
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-primary);
  font-size: var(--font-sm);
  outline: none;
  transition: border-color 0.15s ease;
}
.input::placeholder {
  color: var(--text-tertiary);
}
.input:focus {
  border-color: var(--border-strong);
}
.input.is-error {
  border-color: var(--danger);
}
.input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
