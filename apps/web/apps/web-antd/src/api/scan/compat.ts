import { requestClient } from '#/api/request';

type EntityId = number | string;

function snakeToCamel(key: string) {
  return key.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
}

function camelToSnake(key: string) {
  return key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

/** Convert the copied Kairos page payloads to the RustSet JSON convention. */
export function toInfraPayload(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(toInfraPayload);
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, child]) => [
      snakeToCamel(key),
      toInfraPayload(child),
    ]),
  );
}

/** Keep the upstream page model stable while RustSet returns camelCase. */
export function fromInfraResponse<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((child) => fromInfraResponse(child)) as T;
  }
  if (!value || typeof value !== 'object') return value;

  const result = Object.fromEntries(
    Object.entries(value).map(([key, child]) => [
      camelToSnake(key),
      fromInfraResponse(child),
    ]),
  ) as Record<string, unknown>;
  // Kairos uses created_at/updated_at; the local schema uses create_time/update_time.
  result.created_at ??= result.create_time;
  result.updated_at ??= result.update_time;
  return result as T;
}

export async function infraList<T>(resource: string): Promise<T[]> {
  const rows = await requestClient.get<T[]>(`/infra/${resource}/list`);
  return fromInfraResponse(rows);
}

export async function infraGet<T>(resource: string, id: EntityId): Promise<T> {
  const row = await requestClient.get<T>(`/infra/${resource}/get`, {
    params: { id },
  });
  return fromInfraResponse(row);
}

export function infraCreate(resource: string, data: unknown) {
  return requestClient.post(`/infra/${resource}/create`, toInfraPayload(data));
}

export function infraUpdate(resource: string, id: EntityId, data: unknown) {
  return requestClient.put(
    `/infra/${resource}/update`,
    toInfraPayload({ ...(data as Record<string, unknown>), id }),
  );
}

export function infraDelete(resource: string, id: EntityId) {
  return requestClient.delete(`/infra/${resource}/delete`, { params: { id } });
}

export async function infraPageList<T>(resource: string): Promise<T[]> {
  const page = await requestClient.get<{ list: T[] }>(`/infra/${resource}/page`, {
    params: { pageNo: 1, pageSize: 200 },
  });
  return fromInfraResponse(page.list);
}
