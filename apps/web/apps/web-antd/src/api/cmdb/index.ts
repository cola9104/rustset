import { requestClient } from '#/api/request';

export interface CmdbAttribute {
  id?: number;
  modelId?: number;
  name: string;
  code: string;
  attrType: string;
  required: boolean;
  choices?: { label: string; value: string }[] | null;
  defaultValue?: any;
  showInList: boolean;
  sort: number;
}

export interface CmdbModel {
  id?: number;
  name: string;
  code: string;
  description?: string;
  icon?: string;
  uniqueKey?: string;
  sort: number;
  status?: number;
  instanceCount?: number;
  attributeCount?: number;
}

export interface CmdbInstance {
  id: number;
  modelId: number;
  attributes: Record<string, any>;
  createTime?: string;
  updateTime?: string;
}

export const ATTR_TYPES = [
  { value: 'text', label: '文本' },
  { value: 'textarea', label: '长文本' },
  { value: 'number', label: '整数' },
  { value: 'float', label: '浮点数' },
  { value: 'bool', label: '布尔' },
  { value: 'date', label: '日期' },
  { value: 'datetime', label: '日期时间' },
  { value: 'select', label: '单选' },
  { value: 'multi_select', label: '多选' },
  { value: 'link', label: '链接' },
  { value: 'json', label: 'JSON' },
  { value: 'password', label: '密码' },
] as const;

// ---------- model ----------
export function getModelList() {
  return requestClient.get<CmdbModel[]>('/cmdb/model/list');
}
export function getModelPage(params: { pageNo?: number; pageSize?: number; keyword?: string }) {
  return requestClient.get<{ list: CmdbModel[]; total: number }>('/cmdb/model/page', { params });
}
export function getModel(id: number) {
  return requestClient.get<CmdbModel>('/cmdb/model/get', { params: { id } });
}
export function createModel(data: CmdbModel) {
  return requestClient.post('/cmdb/model/create', data);
}
export function updateModel(data: CmdbModel & { id: number }) {
  return requestClient.put('/cmdb/model/update', data);
}
export function deleteModel(id: number) {
  return requestClient.delete('/cmdb/model/delete', { params: { id } });
}

// ---------- attribute ----------
export function getAttributesByModel(modelId: number) {
  return requestClient.get<CmdbAttribute[]>('/cmdb/attribute/list-by-model', {
    params: { modelId },
  });
}
export function createAttribute(data: CmdbAttribute) {
  return requestClient.post('/cmdb/attribute/create', data);
}
export function updateAttribute(data: CmdbAttribute & { id: number }) {
  return requestClient.put('/cmdb/attribute/update', data);
}
export function deleteAttribute(id: number) {
  return requestClient.delete('/cmdb/attribute/delete', { params: { id } });
}

// ---------- instance ----------
export function getInstancePage(params: {
  modelId: number;
  pageNo?: number;
  pageSize?: number;
  keyword?: string;
}) {
  return requestClient.get<{ list: CmdbInstance[]; total: number }>('/cmdb/instance/page', {
    params,
  });
}
export function createInstance(modelId: number, attributes: Record<string, any>) {
  return requestClient.post('/cmdb/instance/create', { modelId, attributes });
}
export function updateInstance(id: number, attributes: Record<string, any>) {
  return requestClient.put('/cmdb/instance/update', { id, attributes });
}
export function deleteInstance(id: number) {
  return requestClient.delete('/cmdb/instance/delete', { params: { id } });
}
export function deleteInstances(ids: number[]) {
  return requestClient.delete('/cmdb/instance/delete-list', { params: { ids: ids.join(',') } });
}

// ---------- relation ----------
export function getRelationsByInstance(instanceId: number) {
  return requestClient.get<
    { id: number; sourceId: number; targetId: number; relation: string }[]
  >('/cmdb/relation/list-by-instance', { params: { instanceId } });
}
export function bindRelation(sourceId: number, targetId: number, relation?: string) {
  return requestClient.post('/cmdb/relation/bind', { sourceId, targetId, relation });
}
export function unbindRelation(id: number) {
  return requestClient.delete('/cmdb/relation/unbind', { params: { id } });
}
