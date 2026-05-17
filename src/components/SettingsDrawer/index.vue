<script setup lang="ts">
import { computed, reactive } from "vue";
import { open } from "@tauri-apps/plugin-shell";
import {
  RadioButton,
  RadioGroup,
  Space,
  Tooltip,
  Modal,
  Divider,
  Badge,
  Button,
  Typography,
  TypographyTitle,
  TypographyParagraph,
  TypographyLink,
} from "ant-design-vue";
import { InfoCircleOutlined } from "@ant-design/icons-vue";
import { use_config_store, PlayMode } from "@/store/config";
import { app_version } from "@/config";
import { faqData } from "./faq";

const config_store = use_config_store();

const state = reactive({ app_info_modal: false, FAQ: false, tip: false });
const version_update = computed(() => {
  const latest_version = config_store.app_info.app_version ?? app_version;
  return app_version !== latest_version;
});

const to_download = () => {
  open(config_store.app_info.app_download_link);
};

const modal_width = 350;
</script>

<template>
  <div>
    模式选择：
    <Space>
      <Tooltip color="cyan">
        <template #title>
          聆听模式仅可以听，如需脚本演奏请切换为演奏模式
        </template>
        <InfoCircleOutlined />
      </Tooltip>
      <RadioGroup v-model:value="config_store.play_mode" size="small">
        <RadioButton :value="PlayMode.listen">聆听</RadioButton>
        <RadioButton :value="PlayMode.script">演奏</RadioButton>
      </RadioGroup>
    </Space>
  </div>

  <Divider />
  <Space direction="vertical">
    <div @click="state.FAQ = true" style="cursor: pointer">常见问题</div>
    <div @click="state.app_info_modal = true" style="cursor: pointer">
      <Space>
        <Badge :dot="version_update">软件信息</Badge>
        <span v-if="version_update" style="color: gray"> 有更新 </span>
      </Space>
    </div>
  </Space>

  <div class="tip" @click="state.tip = true">开发者碎碎念</div>

  <Modal v-model:open="state.FAQ" :footer="false" centered>
    <div class="modal-content">
      <Typography>
        <template v-for="item in faqData" :key="item.id">
          <TypographyTitle :level="5">{{ item.title }}</TypographyTitle>

          <template v-for="(contentItem, index) in item.content" :key="index">
            <template v-if="contentItem.type === 'text'">
              <TypographyParagraph>
                {{ contentItem.value }}
              </TypographyParagraph>
            </template>

            <template v-else-if="contentItem.type === 'link'">
              <TypographyParagraph>
                <TypographyLink :href="contentItem.href" target="_blank">
                  {{ contentItem.value }}
                </TypographyLink>
              </TypographyParagraph>
            </template>

            <template v-else-if="contentItem.type === 'strong'">
              <TypographyParagraph strong>
                {{ contentItem.value }}
              </TypographyParagraph>
            </template>
          </template>
        </template>
      </Typography>
    </div>
  </Modal>

  <Modal
    v-model:open="state.app_info_modal"
    :footer="false"
    :width="modal_width"
    centered
  >
    <div class="modal-content">
      <p>当前版本：{{ app_version }}</p>
      <p>最新版本：{{ config_store.app_info.app_version }}</p>
      <p>版本描述：{{ config_store.app_info.app_version_description }}</p>
      <p>
        前往更新：<Button type="link" @click="to_download">专栏链接</Button>
      </p>
    </div>
  </Modal>
  <Modal v-model:open="state.tip" :footer="false" :width="modal_width" centered>
    <p style="text-align: center">{{ config_store.app_info.announcement }}</p>
  </Modal>
</template>

<style scoped lang="less">
.modal-content {
  height: clamp(200px, 70vh, 900px);
  overflow-y: scroll;
  scrollbar-width: none;
}

div.ant-typography {
  text-indent: 2em;
}

.tip {
  position: absolute;
  bottom: 20px;
  right: 20px;
  cursor: pointer;
  color: gray;
  font-size: small;
}
</style>
