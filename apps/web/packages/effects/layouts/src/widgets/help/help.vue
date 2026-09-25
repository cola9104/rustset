<script lang="ts" setup>
import { $t } from '@vben/locales';

import { useVbenModal } from '@vben-core/popup-ui';
import { Badge } from '@vben-core/shadcn-ui';

import { useMagicKeys, whenever } from '@vueuse/core';

defineOptions({ name: 'Help' });

const keys = useMagicKeys();
whenever(keys['Alt+KeyH']!, () => modalApi.open());

const [Modal, modalApi] = useVbenModal({
  draggable: true,
  overlayBlur: 5,
  footer: false,
  onCancel() {
    modalApi.close();
  },
});
</script>

<template>
  <Modal class="w-1/3" :title="$t('ui.widgets.qa')">
    <div class="space-y-3 p-2 text-sm">
      <p class="text-base font-semibold">RustSet 管理平台</p>
      <p>资产、CMDB、云资源、运维流程和 AI 能力使用同一套租户与权限体系。</p>
      <p>部署、配置与开发说明请查看仓库根目录 README 和 docs 目录。</p>
      <p class="flex items-center pt-2">
        开源许可：<Badge class="ml-2">MIT</Badge>
      </p>
    </div>
  </Modal>
</template>
