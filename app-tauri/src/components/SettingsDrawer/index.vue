<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
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
  message,
} from "ant-design-vue";
import { InfoCircleOutlined } from "@ant-design/icons-vue";
import { use_config_store, PlayMode } from "@/store/config";
import { app_version } from "@/config";
import {
  get_elevation_state,
  restart_as_admin,
  set_launch_as_administrator,
} from "@/util/elevation";
import { faqData } from "./faq";

const config_store = use_config_store();

const state = reactive({ app_info_modal: false, FAQ: false, tip: false });
const is_elevated = ref(false);
const launch_as_administrator = ref(false);

const version_update = computed(() => {
  const latest_version = config_store.app_info.app_version ?? app_version;
  return app_version !== latest_version;
});

const need_restart = computed(
  () => launch_as_administrator.value && !is_elevated.value,
);

const refresh_elevation = async () => {
  try {
    const s = await get_elevation_state();
    is_elevated.value = s.isElevated;
    launch_as_administrator.value = s.launchAsAdministrator;
  } catch {
    /* 非 Tauri 环境（如纯 Web 预览）忽略 */
  }
};

onMounted(refresh_elevation);

const on_set_launch_admin = async (enabled: boolean) => {
  if (launch_as_administrator.value === enabled) return;
  try {
    await set_launch_as_administrator(enabled);
    await refresh_elevation();
    if (enabled && !is_elevated.value) {
      message.info(
        "已开启：请点击下方按钮以管理员重启，或下次启动时自动提升。",
      );
    } else if (!enabled && is_elevated.value) {
      message.info("已关闭：请退出后按普通方式重新打开应用。");
    }
  } catch (e) {
    message.error(e instanceof Error ? e.message : String(e));
  }
};

const on_restart_as_admin = async () => {
  try {
    message.loading("正在以管理员身份重新启动…", 1);
    await restart_as_admin();
  } catch (e) {
    message.error(e instanceof Error ? e.message : String(e));
  }
};

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
          聆听模式仅播放，如需自动演奏请切换为演奏模式
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

  <div class="admin-section">
    <TypographyTitle :level="5" class="section-title">权限与启动</TypographyTitle>
    <TypographyParagraph type="secondary" class="admin-desc">
      如自动演奏未生效，可尝试使用管理员权限运行本软件。
    </TypographyParagraph>
    <div class="admin-kv">
      <span>当前权限</span>
      <span :class="{ 'admin-kv--highlight': is_elevated }">
        {{ is_elevated ? "管理员" : "普通用户" }}
      </span>
    </div>
    <div class="admin-kv">
      <span>下次启动</span>
      <span :class="{ 'admin-kv--highlight': launch_as_administrator }">
        {{ launch_as_administrator ? "以管理员身份" : "普通用户" }}
      </span>
    </div>
    <div class="admin-toggle-label">以管理员身份启动</div>
    <RadioGroup
      :value="launch_as_administrator"
      size="small"
      class="admin-toggle"
      @change="(e) => on_set_launch_admin(e.target.value === true)"
    >
      <RadioButton :value="false">关闭</RadioButton>
      <RadioButton :value="true">开启</RadioButton>
    </RadioGroup>
    <Button
      v-if="need_restart"
      type="primary"
      block
      class="admin-restart"
      @click="on_restart_as_admin"
    >
      立即以管理员重启
    </Button>
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

.section-title {
  margin-bottom: 8px !important;
}

.admin-desc {
  font-size: 12px;
  text-indent: 0 !important;
  margin-bottom: 12px !important;
}

.admin-kv {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  margin-bottom: 8px;
  color: rgba(0, 0, 0, 0.65);
}

.admin-kv--highlight {
  color: #fa2d48;
  font-weight: 600;
}

.admin-toggle-label {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
  margin: 8px 0 6px;
}

.admin-toggle {
  display: flex;
  width: 100%;

  :deep(.ant-radio-button-wrapper) {
    flex: 1;
    text-align: center;
  }
}

.admin-restart {
  margin-top: 12px;
}
</style>
