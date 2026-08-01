<script setup lang="ts">
import { ref } from "vue";
import Button from "./ui/Button.vue";
import Confirm from "./ui/Confirm.vue";
import EmptyState from "./ui/EmptyState.vue";
import Input from "./ui/Input.vue";
import Modal from "./ui/Modal.vue";
import Select from "./ui/Select.vue";
import type { SelectOption } from "./ui/Select.vue";
import Spinner from "./ui/Spinner.vue";
import Table from "./ui/Table.vue";
import type { TableColumn } from "./ui/Table.vue";
import Tag from "./ui/Tag.vue";
import Textarea from "./ui/Textarea.vue";

// 临时占位页：Plan 06 将替换为真实脚本库页面。
// 此处仅用于验收 Plan 04 —— 基础组件 demo。
const name = ref("");
const command = ref("");
const kind = ref("powershell");
const shellOptions: SelectOption[] = [
  { value: "powershell", label: "PowerShell" },
  { value: "cmd", label: "cmd" },
  { value: "bash", label: "bash" },
];

const showModal = ref(false);
const showConfirm = ref(false);
const confirmLoading = ref(false);

function fakeConfirm() {
  confirmLoading.value = true;
  window.setTimeout(() => {
    confirmLoading.value = false;
    showConfirm.value = false;
  }, 600);
}

const columns: TableColumn[] = [
  { key: "name", label: "名称" },
  { key: "type", label: "类型" },
  { key: "status", label: "状态", slot: "status" },
];
const rows: Record<string, unknown>[] = [
  { id: 1, name: "部署", type: "命令", status: "success" },
  { id: 2, name: "巡检", type: "命令", status: "running" },
  { id: 3, name: "清理", type: "文件", status: "timeout" },
];
</script>

<template>
  <div class="demo">
    <h4 class="demo-h">按钮</h4>
    <div class="demo-row">
      <Button>主按钮</Button>
      <Button variant="secondary">次按钮</Button>
      <Button variant="danger">危险</Button>
      <Button variant="ghost">幽灵</Button>
      <Button variant="text">文字</Button>
      <Button :loading="true">加载中</Button>
      <Button disabled>禁用</Button>
      <Button size="sm" variant="secondary">小按钮</Button>
    </div>

    <h4 class="demo-h">输入</h4>
    <div class="demo-row">
      <Input v-model="name" placeholder="名称" style="width: 200px" />
      <Input v-model="name" error placeholder="错误态" style="width: 200px" />
      <Textarea v-model="command" mono :rows="2" placeholder="命令内容" style="width: 260px" />
    </div>

    <h4 class="demo-h">下拉</h4>
    <div class="demo-row">
      <Select v-model="kind" :options="shellOptions" placeholder="选择 Shell" style="width: 200px" />
    </div>

    <h4 class="demo-h">表格 + 标签</h4>
    <Table :columns="columns" :rows="rows" row-key="id">
      <template #status="scope">
        <Tag :tone="String(scope.row.status)" :label="String(scope.row.status)" />
      </template>
    </Table>

    <h4 class="demo-h">弹窗 / 确认 / 加载</h4>
    <div class="demo-row">
      <Button variant="secondary" @click="showModal = true">打开弹窗</Button>
      <Button variant="danger" @click="showConfirm = true">二次确认</Button>
      <Spinner />
    </div>

    <Modal v-model:open="showModal" title="示例弹窗">
      <p>手写 Modal 组件：ESC / 遮罩点击 / 右上角 × 均可关闭。</p>
    </Modal>
    <Confirm
      v-model:open="showConfirm"
      title="删除脚本"
      message="确认删除「示例脚本」？此操作不可撤销。"
      :loading="confirmLoading"
      @confirm="fakeConfirm"
    />

    <h4 class="demo-h">空态</h4>
    <EmptyState title="暂无运行任务" description="从脚本库启动脚本后，这里会显示实时输出。" />
  </div>
</template>

<style scoped>
.demo {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.demo-h {
  font-size: var(--font-sm);
  font-weight: 600;
  color: var(--text-secondary);
  margin-top: var(--space-2);
}
.demo-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}
</style>
