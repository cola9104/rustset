<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { message } from 'ant-design-vue';
import { getCloudZoneList, createCloudZone, updateCloudZone, deleteCloudZone } from '#/api/scan/cloud-platform';
import { useLocalTableSearch } from '../composables/useLocalTableSearch';
interface CloudZone { id?: number; zone_name: string; zone_code: string; description?: string; }
const data = ref<CloudZone[]>([]); const loading = ref(false); const modalVisible = ref(false); const editingId = ref<number | undefined>();
const { filteredRows, searchText } = useLocalTableSearch(data, ['zone_name', 'zone_code', 'description']);
const form = ref<CloudZone>({ zone_name: '', zone_code: '', description: '' });
const columns = [{ title: '资源区名称', dataIndex: 'zone_name' }, { title: '编码', dataIndex: 'zone_code' }, { title: '描述', dataIndex: 'description' }, { title: '操作', key: 'actions', width: 200 }];
async function fetchData() { loading.value = true; try { data.value = await getCloudZoneList() as any; } catch { message.error('加载失败'); } finally { loading.value = false; } }
onMounted(fetchData);
function openCreate() { editingId.value = undefined; form.value = { zone_name: '', zone_code: '', description: '' }; modalVisible.value = true; }
function openEdit(r: CloudZone) { editingId.value = r.id; form.value = { ...r }; modalVisible.value = true; }
async function handleSubmit() { if (!form.value.zone_name.trim()) { message.warning('请输入资源区名称'); return; } if (editingId.value) { await updateCloudZone(editingId.value, form.value as any); message.success('更新成功'); } else { await createCloudZone(form.value as any); message.success('创建成功'); } modalVisible.value = false; fetchData(); }
</script>
<template>
  <Page auto-content-height>
    <div style="padding:16px">
      <a-space :size="24" style="margin-bottom:20px"><a-button v-access:code="['infra:cloud-zone:create']" type="primary" @click="openCreate">新建云资源区</a-button><a-input-search v-model:value="searchText" allow-clear placeholder="搜索名称、编码或描述" style="width:260px" /><a-button @click="fetchData">刷新</a-button></a-space>
      <a-table :columns="columns" :data-source="filteredRows" :loading="loading" row-key="id" size="middle" :pagination="{pageSize:20}">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'actions'">
            <a-space :size="12"><a-button v-access:code="['infra:cloud-zone:update']" size="small" @click="openEdit(record)">编辑</a-button><a-popconfirm title="确认删除?" @confirm="deleteCloudZone(record.id!).then(fetchData)"><a-button v-access:code="['infra:cloud-zone:delete']" size="small" danger>删除</a-button></a-popconfirm></a-space>
          </template>
        </template>
      </a-table>
      <a-modal v-model:open="modalVisible" :title="editingId ? '编辑云资源区' : '新建云资源区'" @ok="handleSubmit" width="500px">
        <a-form layout="vertical">
          <a-form-item label="资源区名称" required><a-input v-model:value="form.zone_name" placeholder="公有云资源区" /></a-form-item>
          <a-form-item label="编码" required><a-input v-model:value="form.zone_code" placeholder="north" /></a-form-item>
          <a-form-item label="描述"><a-textarea v-model:value="form.description" /></a-form-item>
        </a-form>
      </a-modal>
    </div>
  </Page>
</template>
