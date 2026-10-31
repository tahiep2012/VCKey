// VCKeys 3.0.2 - Modern UI Controller
const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: async (cmd, args) => console.log('Mock IPC:', cmd, args) };

let currentSettings = {};

// Tab navigation
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    const targetId = btn.getAttribute('data-tab');
    document.getElementById(targetId).classList.add('active');
  });
});

// Load initial settings
async function loadData() {
  try {
    currentSettings = await invoke('get_settings') || {};

    // Populate Settings UI
    if (currentSettings.mode) document.getElementById('selectMode').value = currentSettings.mode;
    if (currentSettings.input_type) document.getElementById('selectInputType').value = currentSettings.input_type;
    if (currentSettings.charset) document.getElementById('selectCharset').value = currentSettings.charset;
    if (currentSettings.cand_page_size) document.getElementById('selectCandSize').value = currentSettings.cand_page_size.toString();
    
    const fontSize = currentSettings.cand_font_size || 17;
    document.getElementById('inputCandFontSize').value = fontSize;
    highlightFontSizeItem(fontSize);

    document.getElementById('chkFreeMark').checked = currentSettings.vn_free_mark ?? true;
    document.getElementById('chkAutoEnglish').checked = currentSettings.vn_auto_english_restore ?? true;
    document.getElementById('chkSpellingCheck').checked = currentSettings.vn_spelling_check ?? true;
    document.getElementById('chkNewAccentStyle').checked = currentSettings.vn_new_accent_style ?? false;
    document.getElementById('chkSentenceCompose').checked = currentSettings.chinese_sentence_compose ?? true;
    document.getElementById('chkFuzzyPinyin').checked = currentSettings.chinese_fuzzy_pinyin ?? true;
    document.getElementById('chkChineseJianpin').checked = currentSettings.chinese_jianpin ?? true;
    document.getElementById('chkChinesePunctuation').checked = currentSettings.chinese_punctuation ?? true;
    document.getElementById('chkEmoji').checked = currentSettings.chinese_emoji ?? true;
    document.getElementById('chkTraditional').checked = currentSettings.traditional_chinese ?? false;
    document.getElementById('chkShowPinyinTone').checked = currentSettings.chinese_show_pinyin_tone ?? true;
    document.getElementById('chkRunOnStartup').checked = currentSettings.run_on_startup ?? false;
    document.getElementById('chkMinimizeToTray').checked = currentSettings.minimize_to_tray ?? true;
    document.getElementById('chkShowOsd').checked = currentSettings.show_osd_toast ?? true;
    document.getElementById('chkBeepSwitch').checked = currentSettings.beep_on_switch ?? false;

    const theme = currentSettings.theme || 'system';
    if (document.getElementById('selectTheme')) document.getElementById('selectTheme').value = theme;
    applyTheme(theme);

    updateModeBadge(currentSettings.mode || 'Vietnamese');
  } catch (err) {
    console.error('Error loading data:', err);
  }
}

function applyTheme(theme) {
  if (theme === 'dark') {
    document.body.classList.add('dark-theme');
  } else if (theme === 'light') {
    document.body.classList.remove('dark-theme');
  } else {
    // system
    const isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (isDark) {
      document.body.classList.add('dark-theme');
    } else {
      document.body.classList.remove('dark-theme');
    }
  }
}

window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  const currentTheme = document.getElementById('selectTheme')?.value || 'system';
  if (currentTheme === 'system') {
    applyTheme('system');
  }
});

function highlightFontSizeItem(val) {
  document.querySelectorAll('#dropdownFontSize .combobox-item').forEach(item => {
    if (parseInt(item.getAttribute('data-value')) === parseInt(val)) {
      item.classList.add('selected');
    } else {
      item.classList.remove('selected');
    }
  });
}

function updateModeBadge(mode) {
  const badge = document.getElementById('currentModeBadge');
  if (mode === 'Vietnamese') {
    badge.className = 'mode-badge vn';
    badge.textContent = '🇻🇳 Tiếng Việt';
  } else if (mode === 'Chinese') {
    badge.className = 'mode-badge cn';
    badge.textContent = '🇨🇳 Tiếng Trung';
  } else {
    badge.className = 'mode-badge en';
    badge.textContent = '🇬🇧 Tiếng Anh';
  }
}

// Exit button
document.getElementById('btnExit').addEventListener('click', async () => {
  try {
    await invoke('exit_app');
  } catch (_) {
    window.close();
  }
});

// Font size editable combobox controller
const fontSizeInput = document.getElementById('inputCandFontSize');
const fontSizeDropdown = document.getElementById('dropdownFontSize');
const btnToggleFontSize = document.getElementById('btnToggleFontSize');

btnToggleFontSize.addEventListener('click', (e) => {
  e.stopPropagation();
  fontSizeDropdown.classList.toggle('open');
});

fontSizeInput.addEventListener('click', (e) => {
  e.stopPropagation();
  fontSizeDropdown.classList.add('open');
});

fontSizeInput.addEventListener('input', () => {
  highlightFontSizeItem(fontSizeInput.value);
});

fontSizeInput.addEventListener('blur', () => {
  let val = parseInt(fontSizeInput.value) || 17;
  if (val < 8) val = 8;
  if (val > 48) val = 48;
  fontSizeInput.value = val;
  highlightFontSizeItem(val);
});

document.querySelectorAll('#dropdownFontSize .combobox-item').forEach(item => {
  item.addEventListener('click', (e) => {
    e.stopPropagation();
    const val = item.getAttribute('data-value');
    fontSizeInput.value = val;
    highlightFontSizeItem(val);
    fontSizeDropdown.classList.remove('open');
  });
});

document.addEventListener('click', (e) => {
  if (!document.getElementById('fontSizeCombobox').contains(e.target)) {
    fontSizeDropdown.classList.remove('open');
  }
});

// Save settings button
document.getElementById('btnApply').addEventListener('click', async () => {
  let customFontSize = parseInt(fontSizeInput.value) || 17;
  if (customFontSize < 8) customFontSize = 8;
  if (customFontSize > 48) customFontSize = 48;
  fontSizeInput.value = customFontSize;

  const updatedSettings = {
    ...currentSettings,
    mode: document.getElementById('selectMode').value,
    input_type: document.getElementById('selectInputType').value,
    charset: document.getElementById('selectCharset').value,
    cand_page_size: parseInt(document.getElementById('selectCandSize').value) || 5,
    cand_font_size: customFontSize,
    vn_free_mark: document.getElementById('chkFreeMark').checked,
    vn_auto_english_restore: document.getElementById('chkAutoEnglish').checked,
    vn_spelling_check: document.getElementById('chkSpellingCheck').checked,
    vn_new_accent_style: document.getElementById('chkNewAccentStyle').checked,
    chinese_sentence_compose: document.getElementById('chkSentenceCompose').checked,
    chinese_fuzzy_pinyin: document.getElementById('chkFuzzyPinyin').checked,
    chinese_jianpin: document.getElementById('chkChineseJianpin').checked,
    chinese_punctuation: document.getElementById('chkChinesePunctuation').checked,
    chinese_emoji: document.getElementById('chkEmoji').checked,
    traditional_chinese: document.getElementById('chkTraditional').checked,
    chinese_show_pinyin_tone: document.getElementById('chkShowPinyinTone').checked,
    run_on_startup: document.getElementById('chkRunOnStartup').checked,
    minimize_to_tray: document.getElementById('chkMinimizeToTray').checked,
    show_osd_toast: document.getElementById('chkShowOsd').checked,
    beep_on_switch: document.getElementById('chkBeepSwitch').checked,
    theme: document.getElementById('selectTheme')?.value || 'system',
  };

  try {
    await invoke('save_settings', { settings: updatedSettings });
    updateModeBadge(updatedSettings.mode);
    applyTheme(updatedSettings.theme);
    
    const btn = document.getElementById('btnApply');
    const originalText = btn.textContent;
    btn.textContent = '✓ Đã lưu cài đặt';
    btn.style.backgroundColor = '#16a34a';
    setTimeout(() => {
      btn.textContent = originalText;
      btn.style.backgroundColor = '';
    }, 1500);
  } catch (err) {
    console.error('Lỗi khi lưu cấu hình:', err);
  }
});

// Theme change dropdown listener
document.getElementById('selectTheme')?.addEventListener('change', async (e) => {
  const theme = e.target.value;
  applyTheme(theme);
  try {
    await invoke('set_theme', { theme });
  } catch (err) {
    console.error('Lỗi khi đổi giao diện:', err);
  }
});

// Mode change dropdown listener
document.getElementById('selectMode').addEventListener('change', async (e) => {
  const mode = e.target.value;
  await invoke('set_mode', { mode });
  updateModeBadge(mode);
});

// Click badge to cycle mode
document.getElementById('currentModeBadge').addEventListener('click', async () => {
  const currentMode = await invoke('get_current_mode') || 'Vietnamese';
  const nextMode = currentMode === 'Vietnamese' ? 'Chinese' : (currentMode === 'Chinese' ? 'English' : 'Vietnamese');
  await invoke('set_mode', { mode: nextMode });
  document.getElementById('selectMode').value = nextMode;
  updateModeBadge(nextMode);
});

// Listen for mode-changed event from backend (when Ctrl+Shift or Alt+Shift is pressed globally)
if (window.__TAURI__ && window.__TAURI__.event) {
  window.__TAURI__.event.listen('mode-changed', (event) => {
    const mode = event.payload;
    if (mode) {
      updateModeBadge(mode);
      const selMode = document.getElementById('selectMode');
      if (selMode && selMode.value !== mode) {
        selMode.value = mode;
      }
    }
  });

  window.__TAURI__.event.listen('settings-updated', (event) => {
    const settings = event.payload;
    if (settings) {
      const chkTrad = document.getElementById('chkTraditional');
      if (chkTrad && settings.traditional_chinese !== undefined) {
        chkTrad.checked = settings.traditional_chinese;
      }
    }
  });
}

// Sync mode whenever window becomes visible or receives focus
document.addEventListener('visibilitychange', async () => {
  if (!document.hidden) {
    try {
      const liveMode = await invoke('get_current_mode');
      if (liveMode) {
        updateModeBadge(liveMode);
        if (document.getElementById('selectMode').value !== liveMode) {
          document.getElementById('selectMode').value = liveMode;
        }
      }
    } catch (_) {}
  }
});

// Sync mode immediately when window receives focus
window.addEventListener('focus', async () => {
  try {
    const liveMode = await invoke('get_current_mode');
    if (liveMode) {
      updateModeBadge(liveMode);
      if (document.getElementById('selectMode').value !== liveMode) {
        document.getElementById('selectMode').value = liveMode;
      }
    }
  } catch (_) {}
});

// Periodic real-time sync when window is open and visible (every 150ms)
setInterval(async () => {
  if (!document.hidden) {
    try {
      const liveMode = await invoke('get_current_mode');
      if (liveMode) {
        updateModeBadge(liveMode);
        const sel = document.getElementById('selectMode');
        if (sel && sel.value !== liveMode) {
          sel.value = liveMode;
        }
      }
    } catch (_) {}
  }
}, 150);

// Initialize on DOM load
window.addEventListener('DOMContentLoaded', loadData);



