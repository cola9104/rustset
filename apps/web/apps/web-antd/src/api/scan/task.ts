import { requestClient } from '#/api/request';

import { infraCreate, infraDelete, infraGet, infraList, toInfraPayload } from './compat';

export namespace ScanTaskApi {
  export interface ScanTask {
    id: string; scanner_id?: string; name: string; target: string; status: string; start_time?: string;
    end_time?: string; found_assets: number; found_risks: number;
    port_policy: string; domain_brute: boolean; service_detection: boolean;
    os_detection: boolean; site_identify: boolean;
  }
}
export function getTaskList() { return infraList<ScanTaskApi.ScanTask>('task'); }
export function getTask(id: string) { return infraGet<ScanTaskApi.ScanTask>('task', id); }
export function createTask(data: any) { return infraCreate('task', data); }
export function deleteTask(id: string) { return infraDelete('task', id); }
export function executeScan(data: { target_ip: string; ports: number[] }) { return requestClient.post('/infra/task/trigger-scan', toInfraPayload(data)); }
