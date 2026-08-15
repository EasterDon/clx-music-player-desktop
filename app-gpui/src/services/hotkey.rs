use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

#[derive(Clone, Copy)]
pub struct HotkeyIds {
    pub f1: u32,
    pub f2: u32,
}

// 演奏模式 F1/F2；注销时需传入注册时的 HotKey 实例。
pub struct RegisteredScriptHotkeys {
    f1: HotKey,
    f2: HotKey,
}

impl RegisteredScriptHotkeys {
    pub fn ids(&self) -> HotkeyIds {
        HotkeyIds {
            f1: self.f1.id(),
            f2: self.f2.id(),
        }
    }
}

pub fn try_init() -> Result<GlobalHotKeyManager, String> {
    GlobalHotKeyManager::new().map_err(|e| format!("全局快捷键初始化失败: {e}"))
}

pub fn register_script_hotkeys(
    manager: &GlobalHotKeyManager,
) -> Result<RegisteredScriptHotkeys, String> {
    let f1 = HotKey::new(None, Code::F1);
    let f2 = HotKey::new(None, Code::F2);
    manager
        .register(f1)
        .map_err(|e| format!("注册 F1: {e}"))?;
    manager
        .register(f2)
        .map_err(|e| format!("注册 F2: {e}"))?;
    Ok(RegisteredScriptHotkeys { f1, f2 })
}

pub fn unregister_script_hotkeys(
    manager: &GlobalHotKeyManager,
    keys: &RegisteredScriptHotkeys,
) -> Result<(), String> {
    manager
        .unregister(keys.f1)
        .map_err(|e| format!("注销 F1: {e}"))?;
    manager
        .unregister(keys.f2)
        .map_err(|e| format!("注销 F2: {e}"))?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum HotkeyKind {
    F1,
    F2,
}

// 排空通道内已按下事件，避免轮询丢键。
pub fn poll_hotkeys(ids: &HotkeyIds) -> Vec<HotkeyKind> {
    let mut out = Vec::new();
    while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.state != HotKeyState::Pressed {
            continue;
        }
        if event.id == ids.f1 {
            out.push(HotkeyKind::F1);
        } else if event.id == ids.f2 {
            out.push(HotkeyKind::F2);
        }
    }
    out
}
