import {
  infraCreate,
  infraDelete,
  infraGet,
  infraList,
  infraUpdate,
} from './compat';

export interface NetworkPolicy {
  id?: number;
  firewall_name: string;
  destination_organization: string;
  destination_project: string;
  source_organization: string;
  source_project: string;
  source_security_zone: string;
  source_ip: string;
  destination_security_zone: string;
  destination_ip: string;
  service_port: string;
  applicant: string;
  application_date: string;
  traffic_direction: string;
  action: string;
  implementer?: string;
  implementation_date?: string;
  delivery_date?: string;
  create_time?: string;
  update_time?: string;
}

const resource = 'network-policy';

export function getNetworkPolicyList() {
  return infraList<NetworkPolicy>(resource);
}

export function getNetworkPolicy(id: number) {
  return infraGet<NetworkPolicy>(resource, id);
}

export function createNetworkPolicy(data: NetworkPolicy) {
  return infraCreate(resource, data);
}

export function updateNetworkPolicy(id: number, data: NetworkPolicy) {
  return infraUpdate(resource, id, data);
}

export function deleteNetworkPolicy(id: number) {
  return infraDelete(resource, id);
}
