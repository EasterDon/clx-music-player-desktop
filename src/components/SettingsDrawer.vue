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

const config_store = use_config_store();

const state = reactive({ app_info_modal: false, FAQ: false });
const version_update = computed(
  () => app_version !== config_store.app_info.app_version
);

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

  <Modal v-model:open="state.FAQ" :footer="false" centered>
    <div class="modal-content">
      <Typography>
        <TypographyTitle :level="5">为什么一定要管理员权限启动</TypographyTitle>
        <TypographyParagraph>
          目前一梦江湖PC版（非Mumu模拟器）是由管理员权限启动的，如果软件不以管理员权限运行，脚本则无法在游戏中运行。
        </TypographyParagraph>
        <TypographyTitle :level="5">软件会趁机收集我的隐私吗</TypographyTitle>
        <TypographyParagraph>
          不会，开发者拿你隐私又没什么用，而且软件已经开源在了github（包括前后端代码）链接可以去专栏查看。
        </TypographyParagraph>
        <TypographyParagraph>
          如果你还是怀疑开发者编译好分发的软件带后台，可以自行下载源代码进行搭建与编译。
        </TypographyParagraph>
        <TypographyTitle :level="5">为什么曲子更新这么慢</TypographyTitle>
        <TypographyParagraph>
          _(:з」∠)_这个······开发者更新曲子比较随缘，大部分情况都是好友和用户问开发者做谱才会去做，
          如果你有什么想做的谱子可以在专栏留言或者私信（但是找不到简谱的话开发者可能做不了），
          当然如果你自己做好了字母谱想来分享给其他用户也是强烈欢迎的。
        </TypographyParagraph>
        <TypographyTitle :level="5">软件以后会收费吗</TypographyTitle>
        <TypographyParagraph>
          不会，目前主要花销是后台云服务器费用与域名地址费用，开支来讲不算大，开发者可以维持住，
          你可以给开发者视频投点币什么的，毕竟开发者努力升LV6中。
        </TypographyParagraph>
        <TypographyParagraph>
          什么？你说觉得软件做的不错？一定想投喂？那太好了，客官请点这里(´･ω･`)σ
          <TypographyLink
            href="https://www.bilibili.com/video/BV1GJ411x7h7"
            target="_blank"
          >
            投喂
          </TypographyLink>
        </TypographyParagraph>
        <TypographyTitle :level="5">发现了软件的bug</TypographyTitle>
        <TypographyParagraph>
          快点联系我！快点联系我！快点联系我！快点联系我！快点联系我！快点联系我！快点联系我！
        </TypographyParagraph>
        <TypographyParagraph strong> 我让你快点联系我！ </TypographyParagraph>
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
</style>
