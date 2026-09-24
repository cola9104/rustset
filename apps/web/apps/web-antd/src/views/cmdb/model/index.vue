<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { Page } from '@vben/common-ui';
import { message } from 'ant-design-vue';
import {
  ATTR_TYPES,
  createAttribute,
  createModel,
  deleteAttribute,
  deleteModel,
  getAttributesByModel,
  getModelList,
  updateAttribute,
  updateModel,
} from '#/api/cmdb';
import type { CmdbAttribute, CmdbModel } from '#/api/cmdb';

const models = ref<CmdbModel[]>([]);
const loading = ref(false);
const drawerVisible = ref(false);
const editing = ref<CmdbModel | null>(null);
const modelForm = ref<CmdbModel>(defaultModel());

// 属性编辑器状态
const attributes = ref<CmdbAttribute[]>([]);
const attrDrawerVisible = ref(false);
const currentModel = ref<CmdbModel | null>(null);
const newAttr = ref<CmdbAttribute>(defaultAttr());
const choicesText = ref('');

function defaultModel(): CmdbModel {
  return { name: '', code: '', description: '', icon: '', uniqueKey: '', sort: 0 };
}
function defaultAttr(): CmdbAttribute {
  return { name: '', code: '', attrType: 'text', required: false, showInList: true, sort: 0 };
}

async function fetchModels() {
  loading.value = true;
  try {
    models.value = (await getModelList()) as any;
  } catch {
    message.error('加载模型失败');
  } finally {
    loading.value = false;
  }
}
onMounted(fetchModels);

function openCreate() {
  editing.value = null;
  modelForm.value = defaultModel();
  drawerVisible.value = true;
}
function openEdit(row: CmdbModel) {
  editing.value = row;
  modelForm.value = { ...row };
  drawerVisible.value = true;
}
async function submitModel() {
  try {
    if (editing.value) {
      await updateModel({ ...modelForm.value, id: editing.value.id! });
      message.success('模型已更新');
    } else {
      await createModel(modelForm.value);
      message.success('模型已创建，接下来请在属性管理中定义字段');
    }
    drawerVisible.value = false;
    fetchModels();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  }
}
async function removeModel(row: CmdbModel) {
  try {
    await deleteModel(row.id!);
    message.success('已删除');
    fetchModels();
  } catch (error: any) {
    message.error(error?.message || '删除失败');
  }
}

// ---------- 属性管理 ----------
function parseChoices(text: string): { label: string; value: string }[] | null {
  const items = text
    .split(/[\n,，]/)
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => {
      const separator = item.search(/[:：]/);
      const label = separator >= 0 ? item.slice(0, separator).trim() : item;
      const value = separator >= 0 ? item.slice(separator + 1).trim() : '';
      return { label, value: value || label };
    });
  return items.length ? items : null;
}
function choicesToText(choices: CmdbAttribute['choices']): string {
  if (!choices?.length) return '';
  return choices.map((item) => `${item.label}:${item.value}`).join('\n');
}

async function openAttributes(row: CmdbModel) {
  currentModel.value = row;
  attrDrawerVisible.value = true;
  attributes.value = (await getAttributesByModel(row.id!)) as any;
}
async function reloadAttributes() {
  if (currentModel.value?.id) {
    attributes.value = (await getAttributesByModel(currentModel.value.id)) as any;
  }
}
async function addAttribute() {
  if (!currentModel.value?.id) return;
  const attr = { ...newAttr.value, modelId: currentModel.value.id };
  if (attr.attrType === 'select' || attr.attrType === 'multi_select') {
    attr.choices = parseChoices(choicesText.value);
    if (!attr.choices) {
      message.warning('单选/多选属性需要至少一个选项（格式：标签:值，逗号或换行分隔）');
      return;
    }
  }
  try {
    await createAttribute(attr);
    message.success('属性已添加');
    newAttr.value = defaultAttr();
    choicesText.value = '';
    reloadAttributes();
    fetchModels();
  } catch (error: any) {
    message.error(error?.message || '添加失败');
  }
}
async function saveAttribute(row: CmdbAttribute) {
  try {
    await updateAttribute(row as any);
    message.success(`属性 ${row.name} 已保存`);
    fetchModels();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  }
}
async function removeAttribute(row: CmdbAttribute) {
  try {
    await deleteAttribute(row.id!);
    message.success('属性已删除');
    reloadAttributes();
    fetchModels();
  } catch (error: any) {
    message.error(error?.message || '删除失败');
  }
}

const attrTypeOptions = ATTR_TYPES;
const needsChoices = computed(
  () => newAttr.value.attrType === 'select' || newAttr.value.attrType === 'multi_select',
);

const columns = [
  { title: '模型名称', dataIndex: 'name', key: 'name', width: 150 },
  { title: '编码', dataIndex: 'code', key: 'code', width: 130 },
  { title: '唯一键', dataIndex: 'uniqueKey', key: 'uniqueKey', width: 110 },
  { title: '属性数', dataIndex: 'attributeCount', key: 'attributeCount', width: 80 },
  { title: '实例数', dataIndex: 'instanceCount', key: 'instanceCount', width: 80 },
  { title: '说明', dataIndex: 'description', key: 'description', ellipsis: true },
  { title: '排序', dataIndex: 'sort', key: 'sort', width: 70 },
  { title: '操作', key: 'actions', width: 260, fixed: 'right' },
];
</script>

<template>
  <Page auto-content-height>
    <div style="padding: 16px">
      <a-page-header
        title="模型管理"
        sub-title="自定义 CMDB 模型与属性（参考 veops/cmdb）"
        style="margin-bottom: 16px; padding: 0"
      />
      <a-space :size="16" style="margin-bottom: 16px">
        <a-button v-access:code="['cmdb:model:create']" type="primary" @click="openCreate">
          新建模型
        </a-button>
        <a-button @click="fetchModels">刷新</a-button>
      </a-space>
      <a-table
        :columns="columns"
        :data-source="models"
        :loading="loading"
        row-key="id"
        size="middle"
        :pagination="{ pageSize: 20 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <Icon :icon="record.icon || 'lucide:box'" style="margin-right: 6px; color: var(--ant-color-primary)" />
            {{ record.name }}
          </template>
          <template v-if="column.key === 'uniqueKey'">
            <a-tag v-if="record.uniqueKey" color="blue">{{ record.uniqueKey }}</a-tag>
            <span v-else>-</span>
          </template>
          <template v-if="column.key === 'actions'">
            <a-space>
              <a-button
                v-access:code="['cmdb:model:query']"
                type="link"
                size="small"
                @click="openAttributes(record)"
              >
                属性管理
              </a-button>
              <a-button
                v-access:code="['cmdb:model:update']"
                type="link"
                size="small"
                @click="openEdit(record)"
              >
                编辑
              </a-button>
              <a-popconfirm
                title="确认删除该模型？（需先清空其实例）"
                ok-text="删除"
                ok-type="danger"
                cancel-text="取消"
                @confirm="removeModel(record)"
              >
                <a-button v-access:code="['cmdb:model:delete']" type="link" size="small" danger>
                  删除
                </a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>

      <!-- 模型编辑 -->
      <a-modal
        v-model:open="drawerVisible"
        :title="editing ? '编辑模型' : '新建模型'"
        @ok="submitModel"
        width="560px"
      >
        <a-form v-if="modelForm" layout="vertical">
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="模型名称" required>
                <a-input v-model:value="modelForm.name" placeholder="例如：物理服务器" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="模型编码" required>
                <a-input
                  v-model:value="modelForm.code"
                  :disabled="!!editing"
                  placeholder="例如：server（小写字母/数字/下划线）"
                />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="唯一键属性编码" extra="用于识别同一模型下的唯一实例，如 hostname / sn">
                <a-input v-model:value="modelForm.uniqueKey" placeholder="例如：hostname" />
              </a-form-item>
            </a-col>
            <a-col :span="6">
              <a-form-item label="图标">
                <a-input v-model:value="modelForm.icon" placeholder="lucide:server" />
              </a-form-item>
            </a-col>
            <a-col :span="6">
              <a-form-item label="排序">
                <a-input-number v-model:value="modelForm.sort" :min="0" style="width: 100%" />
              </a-form-item>
            </a-col>
            <a-col :span="24">
              <a-form-item label="说明">
                <a-textarea v-model:value="modelForm.description" :rows="2" />
              </a-form-item>
            </a-col>
          </a-row>
        </a-form>
      </a-modal>

      <!-- 属性管理抽屉 -->
      <a-drawer
        v-model:open="attrDrawerVisible"
        :title="`属性管理 · ${currentModel?.name ?? ''}`"
        width="860"
      >
        <a-alert
          type="info"
          show-icon
          style="margin-bottom: 16px"
          message="属性定义了该模型实例的字段结构：类型、必填、选项与默认值。实例只能填写已定义的属性。"
        />
        <a-card size="small" title="新增属性" style="margin-bottom: 16px">
          <a-row :gutter="12">
            <a-col :span="5"><a-input v-model:value="newAttr.name" placeholder="属性名称" /></a-col>
            <a-col :span="5"><a-input v-model:value="newAttr.code" placeholder="属性编码" /></a-col>
            <a-col :span="4">
              <a-select v-model:value="newAttr.attrType" :options="attrTypeOptions as any" />
            </a-col>
            <a-col :span="2" style="text-align: center"><a-checkbox v-model:checked="newAttr.required">必填</a-checkbox></a-col>
            <a-col :span="2" style="text-align: center"><a-checkbox v-model:checked="newAttr.showInList">列表</a-checkbox></a-col>
            <a-col :span="2"><a-input-number v-model:value="newAttr.sort" :min="0" style="width: 100%" placeholder="排序" /></a-col>
            <a-col :span="4">
              <a-button v-access:code="['cmdb:attribute:create']" type="primary" @click="addAttribute">添加</a-button>
            </a-col>
          </a-row>
          <a-textarea
            v-if="needsChoices"
            v-model:value="choicesText"
            :rows="2"
            style="margin-top: 8px"
            placeholder="选项（每行一个，格式 标签:值，例如 核心:core）"
          />
        </a-card>
        <a-table
          :columns="[
            { title: '属性名称', key: 'name', width: 150 },
            { title: '编码', key: 'code', width: 130 },
            { title: '类型', key: 'attrType', width: 120 },
            { title: '必填', key: 'required', width: 70 },
            { title: '列表显示', key: 'showInList', width: 90 },
            { title: '选项', key: 'choices', ellipsis: true },
            { title: '排序', key: 'sort', width: 70 },
            { title: '操作', key: 'actions', width: 130 },
          ]"
          :data-source="attributes"
          row-key="id"
          size="small"
          :pagination="false"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'name'">
              <a-input v-model:value="record.name" size="small" />
            </template>
            <template v-if="column.key === 'code'">
              <a-tag>{{ record.code }}</a-tag>
            </template>
            <template v-if="column.key === 'attrType'">
              <a-select v-model:value="record.attrType" size="small" :options="attrTypeOptions as any" style="width: 100%" />
            </template>
            <template v-if="column.key === 'required'">
              <a-switch v-model:checked="record.required" size="small" />
            </template>
            <template v-if="column.key === 'showInList'">
              <a-switch v-model:checked="record.showInList" size="small" />
            </template>
            <template v-if="column.key === 'choices'">
              <a-input
                :value="choicesToText(record.choices)"
                size="small"
                placeholder="标签:值 每行一个（仅单选/多选）"
                @change="(e: any) => (record.choices = parseChoices(e.target.value))"
              />
            </template>
            <template v-if="column.key === 'sort'">
              <a-input-number v-model:value="record.sort" size="small" :min="0" style="width: 100%" />
            </template>
            <template v-if="column.key === 'actions'">
              <a-space>
                <a-button v-access:code="['cmdb:attribute:update']" type="link" size="small" @click="saveAttribute(record)">保存</a-button>
                <a-popconfirm title="确认删除该属性？" @confirm="removeAttribute(record)">
                  <a-button v-access:code="['cmdb:attribute:delete']" type="link" size="small" danger>删除</a-button>
                </a-popconfirm>
              </a-space>
            </template>
          </template>
        </a-table>
      </a-drawer>
    </div>
  </Page>
</template>
