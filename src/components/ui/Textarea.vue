<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    minHeight?: string;
    mono?: boolean;
    rows?: number;
    error?: boolean;
    disabled?: boolean;
  }>(),
  { placeholder: "", minHeight: "80px", mono: false, rows: 4, error: false, disabled: false },
);

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();
</script>

<template>
  <div class="field">
    <label v-if="$slots.label" class="field-label"><slot name="label" /></label>
    <textarea
      class="textarea"
      :class="{ mono, 'is-error': error }"
      :value="modelValue"
      :placeholder="placeholder"
      :rows="rows"
      :disabled="disabled"
      :style="{ minHeight }"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
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
.textarea {
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-primary);
  font-size: var(--font-sm);
  line-height: 1.5;
  outline: none;
  resize: vertical;
  transition: border-color 0.15s ease;
}
.textarea::placeholder {
  color: var(--text-tertiary);
}
.textarea:focus {
  border-color: var(--border-strong);
}
.textarea.mono {
  font-family: var(--font-mono);
}
.textarea.is-error {
  border-color: var(--danger);
}
.textarea:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
