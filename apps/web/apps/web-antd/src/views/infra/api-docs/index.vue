<script lang="ts" setup>
import { onActivated, onDeactivated, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAppConfig } from '@vben/hooks';

// The package does not export its standalone asset as a subpath.
import standaloneUrl from '../../../../node_modules/@scalar/api-reference/dist/browser/standalone.js?url';

import { buildApiDocsDocument } from './frame';

defineOptions({ name: 'InfraApiDocs' });

const { apiURL } = useAppConfig(import.meta.env, import.meta.env.PROD);
const active = ref(true);
const frameKey = ref(0);
const documentUrl = new URL(
  `${apiURL.replace(/\/$/, '')}/openapi.json`,
  window.location.href,
).href;
const frameDocument = buildApiDocsDocument(
  documentUrl,
  new URL(standaloneUrl, window.location.href).href,
);

// A cached route must release the embedded app while it is inactive.
onActivated(() => {
  active.value = true;
});
onDeactivated(() => {
  active.value = false;
});
</script>

<template>
  <Page auto-content-height>
    <div class="api-docs-wrapper">
      <div class="api-docs-toolbar">
        <span>接口文档</span>
        <button type="button" @click="frameKey++">重新加载</button>
      </div>
      <iframe
        v-if="active"
        :key="frameKey"
        class="api-docs-frame"
        title="RustSet API 接口文档"
        :srcdoc="frameDocument"
      />
    </div>
  </Page>
</template>

<style scoped>
.api-docs-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 560px;
  overflow: hidden;
  background: var(--ant-color-bg-container);
  border-radius: 8px;
}

.api-docs-toolbar {
  display: flex;
  flex-shrink: 0;
  justify-content: space-between;
  padding: 12px 16px;
}

.api-docs-toolbar button {
  color: var(--ant-color-primary, #1677ff);
  cursor: pointer;
}

.api-docs-frame {
  flex: 1;
  width: 100%;
  min-height: 500px;
  border: 0;
}
</style>
