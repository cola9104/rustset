import { requestClient } from '#/api/request';

import { fromInfraResponse } from './compat';

export namespace InspectionApi {
  export interface InspectionTask {
    id: string;
    name: string;
    target: string;
    status: string;
    start_time?: string;
    end_time?: string;
    scan_ports: number[];
    total_targets: number;
    completed_targets: number;
    found_assets: number;
    found_risks: number;
    error_message?: string;
  }

  export interface InspectionDifference {
    kind: string;
    port?: number;
    severity: string;
    description: string;
  }

  export interface InspectionRiskRef {
    id: string;
    asset_ip: string;
    port: number;
    severity: string;
    description: string;
    solution?: string;
    status: string;
  }

  export interface InspectionResult {
    id: string;
    task_id: string;
    ip: string;
    registered: boolean;
    baseline_ports: number[] | null;
    open_ports: number[];
    uncertain_ports: number[];
    differences: InspectionDifference[];
    risks: InspectionRiskRef[];
  }

  export interface InspectionBaseline {
    ip: string;
    allowed_ports: number[];
    reason: string;
    updated_by: string;
    update_time?: string;
  }
}

export async function getInspectionList(): Promise<
  InspectionApi.InspectionTask[]
> {
  const rows = await requestClient.get<any[]>('/infra/inspection/list');
  return fromInfraResponse(rows);
}

export async function getInspectionResults(
  taskId: string,
): Promise<InspectionApi.InspectionResult[]> {
  const rows = await requestClient.get<any[]>('/infra/inspection/results', {
    params: { taskId },
  });
  return fromInfraResponse(rows);
}

export async function getInspectionBaseline(
  ip: string,
): Promise<InspectionApi.InspectionBaseline | null> {
  const row = await requestClient.get<any>('/infra/inspection/baseline', {
    params: { ip },
  });
  return fromInfraResponse(row);
}

export function runInspection(data: {
  name?: string;
  targetIps: string[];
  ports: number[];
}) {
  return requestClient.post('/infra/inspection/run', data);
}

export function saveInspectionBaseline(data: {
  allowedPorts: number[];
  ip: string;
  reason: string;
}) {
  return requestClient.put('/infra/inspection/baseline', data);
}
