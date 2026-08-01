<script setup lang="ts">
export interface TableColumn {
  key: string;
  label: string;
  width?: string;
  align?: "left" | "center" | "right";
  /** 单元格插槽名，接收 { row, value } */
  slot?: string;
}

const props = withDefaults(
  defineProps<{
    columns: TableColumn[];
    rows: Record<string, unknown>[];
    rowKey?: string;
  }>(),
  { rowKey: "id" },
);
</script>

<template>
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th
            v-for="c in columns"
            :key="c.key"
            :style="{ width: c.width, textAlign: c.align ?? 'left' }"
          >
            {{ c.label }}
          </th>
        </tr>
      </thead>
      <tbody v-if="rows.length">
        <tr v-for="(row, ri) in rows" :key="String(row[props.rowKey] ?? ri)">
          <td v-for="c in columns" :key="c.key" :style="{ textAlign: c.align ?? 'left' }">
            <slot v-if="c.slot" :name="c.slot" :row="row" :value="row[c.key]" />
            <template v-else>{{ row[c.key] }}</template>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-if="!rows.length" class="table-empty">
      <slot name="empty"><span>暂无数据</span></slot>
    </div>
  </div>
</template>

<style scoped>
.table-wrap {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-panel);
  overflow: hidden;
}
.table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  font-size: var(--font-sm);
}
.table th {
  padding: 8px 12px;
  font-size: var(--font-xs);
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-page);
  border-bottom: 1px solid var(--border);
  text-align: left;
  white-space: nowrap;
}
.table td {
  padding: 7px 12px;
  border-bottom: 1px solid var(--border);
  color: var(--text-primary);
  vertical-align: middle;
  overflow: hidden;
  text-overflow: ellipsis;
}
.table tbody tr:last-child td {
  border-bottom: none;
}
.table tbody tr:nth-child(even) {
  background: var(--bg-page);
}
.table tbody tr:hover {
  background: var(--bg-hover);
}
.table-empty {
  padding: var(--space-8) var(--space-4);
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--font-sm);
}
</style>
