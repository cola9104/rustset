import { infraCreate, infraDelete, infraGet, infraList, infraUpdate } from './compat';

export namespace ScanAssetApi {
  export interface Asset {
    id?: number; name: string; ip: string; zone: string; ports: any[];
    last_scanned?: string; contact_person?: string; contact_phone?: string;
    owner?: string; weight: number; labels: string[]; os?: string; device_type?: string;
    url?: string; language?: string; finding_count?: number; endpoint_count?: number;
  }
}
export async function getAssetList() {
  const rows = await infraList<ScanAssetApi.Asset>('asset');
  return rows.map((row) => ({
    ...row,
    editable: true,
    record_id: row.id,
    source_label: '本地资产',
    source_type: 'scan',
    status: 'active',
  }));
}
export function getAsset(id: number) { return infraGet<ScanAssetApi.Asset>('asset', id); }
export function createAsset(data: ScanAssetApi.Asset) { return infraCreate('asset', data); }
export function updateAsset(id: number, data: ScanAssetApi.Asset) { return infraUpdate('asset', id, data); }
export function deleteAsset(id: number) { return infraDelete('asset', id); }
