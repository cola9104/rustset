<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAppConfig } from '@vben/hooks';

// 相对路径深引用：包 exports 未暴露 standalone 子路径，?url 资产加载可绕过。
import standaloneUrl from '../../../../node_modules/@scalar/api-reference/dist/browser/standalone.js?url';

// Scalar 官方浏览器独立包：注册 <scalar-api-reference> web component。
// 不走 Vue 组件入口——文档渲染的任何错误都隔离在自定义元素内，
// 不会拖垮宿主 SPA（此前 Vue 组件集成在运行时抛错导致整站白屏）。
// 相对路径深引用：exports 表未暴露该子路径，?url 资产加载可绕过。

defineOptions({ name: 'InfraApiDocs' });

const { apiURL } = useAppConfig(import.meta.env, import.meta.env.PROD);

const container = ref<HTMLDivElement>();
let loader: HTMLScriptElement | null = null;
let bootstrap: HTMLScriptElement | null = null;

onMounted(() => {
  container.value?.replaceChildren();
  // Scalar 约定：容器内 <script id="api-reference" data-url="...">，
  // standalone 脚本加载后自动把该元素升级为文档界面。
  bootstrap = document.createElement('script');
  bootstrap.id = 'api-reference';
  bootstrap.dataset.url = `${apiURL}/openapi.json`;
  container.value?.appendChild(bootstrap);
  loader = document.createElement('script');
  loader.src = standaloneUrl;
  loader.async = true;
  container.value?.appendChild(loader);
});

onUnmounted(() => {
  loader?.remove();
  loader = null;
  bootstrap?.remove();
  bootstrap = null;
  container.value?.replaceChildren();
});
</script>

<template>
  <Page auto-content-height>
    <div class="api-docs-wrapper">
      <div ref="container" class="scalar-container" />
    </div>
  </Page>
</template>

<style scoped>
.api-docs-wrapper {
  height: 100%;
  padding: 12px;
  background: var(--ant-color-bg-container);
  border-radius: 8px;
}

.scalar-container {
  height: 100%;
}
</style>
