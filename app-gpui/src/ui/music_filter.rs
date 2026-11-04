use crate::model::{ClxState, PlayMode};
use crate::ui::root::RootView;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::prelude::*;
use gpui_kit::*;
use gpui_kit::Subscription;
use crate::services::fkey_hint;

fn filter_placeholder(mode: PlayMode) -> String {
    match mode {
        PlayMode::Listen => "聆听模式仅可播放；可在设置中切换为「演奏」".to_string(),
        PlayMode::Script => {
            format!(
                "{}开始自动演奏，{}停止演奏",
                fkey_hint::f1_label(),
                fkey_hint::f2_label()
            )
        }
    }
}

actions!(music_filter, [FilterClear]);

/// 曲库搜索框。
///
/// 输入、IME、选区与光标由 `gpui_kit::component::input::Input` 负责；本类型只负责
/// 把输入内容同步到 `ClxState::filter`，并在占位文案随演奏模式变化时刷新。
pub struct MusicFilterInput {
    model: Entity<ClxState>,
    weak_root: WeakEntity<RootView>,
    state: Entity<InputState>,
    /// 上一次同步到模型的关键词，避免回环重复写入。
    last_text: String,
    /// 上一次设置的占位文案，用于跟随 `play_mode` 变化。
    placeholder: SharedString,
    /// 必须持有：`Subscription` 被 drop 会取消订阅，导致输入不再同步。
    _input_sub: Subscription,
}

impl MusicFilterInput {
    pub fn new(
        model: Entity<ClxState>,
        weak_root: WeakEntity<RootView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (content, placeholder) = cx.read_entity(&model, |s, _| {
            (s.filter.clone(), SharedString::from(filter_placeholder(s.play_mode)))
        });

        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder.clone())
                .default_value(content.clone())
        });

        // 输入变化 -> 同步进模型。
        let input_sub = cx.subscribe_in(
            &state,
            window,
            |this, _, event: &InputEvent, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.push_filter(cx);
                }
            },
        );

        let mut this = Self {
            model,
            weak_root,
            state,
            last_text: content,
            placeholder,
            _input_sub: input_sub,
        };
        this.refresh_placeholder(window, cx);
        this
    }

    /// 把输入框当前内容同步到模型；内容未变时不做任何事。
    fn push_filter(&mut self, cx: &mut Context<Self>) {
        let text = self.state.read(cx).value().to_string();
        if text == self.last_text {
            return;
        }
        self.last_text = text.clone();
        let model = self.model.clone();
        let _ = cx.update_entity(&model, |st, c| {
            st.set_filter(text);
            let ids: Vec<i64> = st
                .filtered_display
                .iter()
                .take(32)
                .map(|m| m.id)
                .collect();
            st.enqueue_music_cover_fetches(ids);
            c.notify();
        });
        if let Some(root) = self.weak_root.upgrade() {
            let _ = root.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    /// 占位文案随演奏模式变化。
    fn refresh_placeholder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let want = cx.read_entity(&self.model, |s, _| filter_placeholder(s.play_mode));
        if self.placeholder.as_ref() == want {
            return;
        }
        self.placeholder = SharedString::from(want);
        let placeholder = self.placeholder.clone();
        self.state.update(cx, |state, cx| {
            state.set_placeholder(placeholder, window, cx);
        });
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.push_filter(cx);
    }

    fn on_clear(&mut self, _: &FilterClear, window: &mut Window, cx: &mut Context<Self>) {
        self.clear(window, cx);
    }
}

impl Render for MusicFilterInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.refresh_placeholder(window, cx);

        div()
            .flex_1()
            .min_w(px(120.0))
            .max_w(px(420.0))
            .key_context("MusicFilter")
            .on_action(cx.listener(Self::on_clear))
            .child(
                Input::new(&self.state)
                    .cleanable(true)
                    .h(px(34.0)),
            )
    }
}
