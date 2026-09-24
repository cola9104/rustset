<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { Page } from '@vben/common-ui';
import { message } from 'ant-design-vue';
import { requestClient } from '#/api/request';

interface NetZone {
  id?: number;
  name: string;
  parentId: number;
  zoneType: string;
  cidr?: string;
  sort: number;
  description?: string;
  children?: NetZone[];
}

const tree = ref<NetZone[]>([]);
const loading = ref(false);
const modalVisible = ref(false);
const editingId = ref<number>();
const form = ref<NetZone>(defaultForm());
const resolveIp = ref('');
const resolveResult = ref<any>(null);
const identifying = ref(false);

const zoneTypeOptions = [
  { value: 'company', label: '公司' },
  { value: 'subsidiary', label: '子公司' },
  { value: 'department', label: '部门' },
  { value: 'segment', label: '网段' },
];
const zoneTypeLabel: Record<string, string> = {
  company: '公司', subsidiary: '子公司', department: '部门', segment: '网段',
};

function defaultForm(): NetZone {
  return { name: '', parentId: 0, zoneType: 'company', cidr: '', sort: 0, description: '' };
}

async function fetchTree() {
  loading.value = true;
  try {
    const result = (await requestClient.get('/cmdb/net-zone/tree')) as any;
    tree.value = result?.tree ?? [];
  } catch {
    message.error('加载组织网段失败');
  } finally {
    loading.value = false;
  }
}
onMounted(fetchTree);

function collectParents(nodes: NetZone[], depth = 0): { value: number; label: string }[] {
  const options: { value: number; label: string }[] = [];
  for (const node of nodes ?? []) {
    options.push({ value: node.id!, label: `${'　'.repeat(depth)}${node.name}` });
    options.push(...collectParents(node.children ?? [], depth + 1));
  }
  return options;
}
const parentOptions = computed(() => [
  { value: 0, label: '（根节点）' },
  ...collectParents(tree.value),
]);

import { computed } from 'vue';

function openCreate(parentId = 0, zoneType?: string) {
  editingId.value = undefined;
  form.value = { ...defaultForm(), parentId, zoneType: zoneType ?? (parentId ? 'segment' : 'company') };
  modalVisible.value = true;
}
function openEdit(row: NetZone) {
  editingId.value = row.id;
  form.value = { ...row, children: undefined } as NetZone;
  modalVisible.value = true;
}
async function handleSubmit() {
  try {
    if (editingId.value) {
      await requestClient.put('/cmdb/net-zone/update', { ...form.value, id: editingId.value });
      message.success('已更新');
    } else {
      await requestClient.post('/cmdb/net-zone/create', form.value);
      message.success('已创建');
    }
    modalVisible.value = false;
    fetchTree();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  }
}
async function handleDelete(row: NetZone) {
  try {
    await requestClient.delete('/cmdb/net-zone/delete', { params: { id: row.id } });
    message.success('已删除');
    fetchTree();
  } catch (error: any) {
    message.error(error?.message || '删除失败');
  }
}

async function resolve() {
  if (!resolveIp.value.trim()) return;
  try {
    resolveResult.value = (await requestClient.post('/cmdb/net-zone/resolve', {
      ip: resolveIp.value,
    })) as any;
  } catch (error: any) {
    message.error(error?.message || '识别失败');
  }
}

async function identifyAssets() {
  identifying.value = true;
  try {
    const result = (await requestClient.post('/cmdb/net-zone/identify-assets', {})) as any;
    message.success(`归属识别完成：命中 ${result?.matched ?? 0} 台，未匹配 ${result?.unmatched ?? 0} 台`);
  } catch (error: any) {
    message.error(error?.message || '识别失败');
  } finally {
    identifying.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <div style="padding:16px">
      <a-page-header
        title="组织网段"
        sub-title="公司 / 子公司 / 部门 / 网段四级树，资产 IP 按最长前缀自动归属"
        style="margin-bottom:12px;padding:0"
      />
      <a-space :size="16" style="margin-bottom:12px" wrap>
        <a-button v-access:code="['cmdb:net-zone:create']" type="primary" @click="openCreate(0, 'company')">新建公司</a-button>
        <a-button v-access:code="['cmdb:net-zone:update']" :loading="identifying" @click="identifyAssets">按网段识别资产归属</a-button>
      </a-space>

      <a-card size="small" title="IP 归属试算" style="margin-bottom:16px">
        <a-space>
          <a-input v-model:value="resolveIp" placeholder="输入 IP，例如 10.1.2.3" style="width:220px" @press-enter="resolve" />
          <a-button type="primary" @click="resolve">识别</a-button>
          <span v-if="resolveResult">
            <a-tag v-if="resolveResult.matched" color="green">
              {{ resolveResult.netZoneName }}（{{ zoneTypeLabel[resolveResult.zoneType] }}）· 归属单位：{{ resolveResult.organization || '-' }}
            </a-tag>
            <a-tag v-else color="orange">未命中任何网段</a-tag>
          </span>
        </a-space>
      </a-card>

      <a-table
        :columns="[
          { title: '名称', key: 'name' },
          { title: '类型', key: 'zoneType', width: 100 },
          { title: '网段 (CIDR)', key: 'cidr', width: 180 },
          { title: '排序', dataIndex: 'sort', key: 'sort', width: 70 },
          { title: '说明', key: 'description', ellipsis: true },
          { title: '操作', key: 'actions', width: 220, fixed: 'right' },
        ]"
        :data-source="tree"
        :loading="loading"
        row-key="id"
        size="middle"
        :pagination="false"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <Icon :icon="record.zoneType === 'segment' ? 'lucide:network' : 'lucide:building-2'" style="margin-right:6px;color:var(--ant-color-primary)" />
            {{ record.name }}
          </template>
          <template v-if="column.key === 'zoneType'">
            <a-tag :color="record.zoneType === 'segment' ? 'blue' : 'geekblue'">{{ zoneTypeLabel[record.zoneType] }}</a-tag>
          </template>
          <template v-if="column.key === 'cidr'">
            <code v-if="record.cidr">{{ record.cidr }}</code>
            <span v-else>-</span>
          </template>
          <template v-if="column.key === 'description'">{{ record.description || '-' }}</template>
          <template v-if="column.key === 'actions'">
            <a-space>
              <a-button v-access:code="['cmdb:net-zone:create']" type="link" size="small" @click="openCreate(record.id, record.zoneType === 'company' ? 'subsidiary' : 'segment')">添加子级</a-button>
              <a-button v-access:code="['cmdb:net-zone:update']" type="link" size="small" @click="openEdit(record)">编辑</a-button>
              <a-popconfirm title="确认删除？（需先删除子级）" ok-text="删除" ok-type="danger" cancel-text="取消" @confirm="handleDelete(record)">
                <a-button v-access:code="['cmdb:net-zone:delete']" type="link" size="small" danger>删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>

      <a-modal v-model:open="modalVisible" :title="editingId ? '编辑节点' : '新建节点'" @ok="handleSubmit" width="560px">
        <a-form v-if="form" layout="vertical">
          <a-row :gutter="16">
            <a-col :span="12"><a-form-item label="名称" required><a-input v-model:value="form.name" /></a-form-item></a-col>
            <a-col :span="12"><a-form-item label="类型"><a-select v-model:value="form.zoneType" :options="zoneTypeOptions" /></a-form-item></a-col>
            <a-col :span="12"><a-form-item label="上级节点"><a-select v-model:value="form.parentId" :options="parentOptions" /></a-form-item></a-col>
            <a-col :span="12"><a-form-item label="网段 (CIDR)" extra="例如 10.1.0.0/16；仅 IPv4"><a-input v-model:value="form.cidr" /></a-form-item></a-col>
            <a-col :span="6"><a-form-item label="排序"><a-input-number v-model:value="form.sort" :min="0" style="width:100%" /></a-form-item></a-col>
            <a-col :span="24"><a-form-item label="说明"><a-textarea v-model:value="form.description" :rows="2" /></a-form-item></a-col>
          </a-row>
        </a-form>
      </a-modal>
    </div>
  </Page>
</template>
