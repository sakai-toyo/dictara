# Keyboard Shortcuts Configuration - Full Implementation Plan

## Summary

This plan evolves the existing simple trigger key feature into a comprehensive 3-shortcut system with key combinations, runtime hot-swapping, and interactive key capture UI.

**Current State (Already Implemented):**
- Simple `RecordingTrigger` enum (Fn/Control/Option/Command) in [config.rs:19-40](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L19-L40)
- Type-safe `ConfigStore` pattern with get/set/delete methods in [config.rs:160-199](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L160-L199)
- Single trigger passed to keyboard listener in [keyboard_listener.rs:20](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs#L20)
- Space hardcoded as LOCK_MODIFIER for hands-free mode (line 9)
- TriggerKeyStep onboarding component with app restart flow
- Hotkeys preferences page with restart button
- Changes require app restart

**Target State:**
- Three separate configurable shortcuts with key combinations (1-3 keys each)
- Runtime hot-swap (no restart required)
- Interactive key capture UI
- Backward compatible migration from current implementation

---

## User Requirements

### Three Configurable Shortcuts

1. **Push to Record** (default: Fn)
   - Hold to record, release to transcribe
   - Current behavior: single trigger key

2. **Hands-free Start** (default: Fn + Space)
   - Toggle recording on (locked mode)
   - Current: hardcoded as trigger + Space

3. **Hands-free Stop** (default: Fn)
   - Stop hands-free recording
   - Current: same as push-to-record trigger

### Key Features

- **Key Combinations**: Up to 3 keys per shortcut (e.g., Shift+Cmd+R, Fn+Space)
- **Runtime Hot-Swap**: Changes take effect immediately without app restart
- **Key Capture UI**: Press actual keys to record combinations (special handling for Fn key)
- **Migration**: Seamlessly upgrade existing users from simple trigger to 3-shortcut system

---

## Architecture Design

### Backend Data Structures

Add to [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs):

```rust
/// A single key in a shortcut combination
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutKey {
    pub keycode: u32,           // macOS keycode (e.g., 63 for Fn)
    pub label: String,          // Display name (e.g., "Fn", "Space")
}

/// A keyboard shortcut (1-3 keys)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub keys: Vec<ShortcutKey>,
}

impl Shortcut {
    /// Check if this shortcut matches currently pressed keys
    pub fn matches(&self, pressed_keys: &HashSet<u32>) -> bool {
        // Exact match: same count AND all keys present
        self.keys.len() == pressed_keys.len()
            && self.keys.iter().all(|k| pressed_keys.contains(&k.keycode))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() || self.keys.len() > 3 {
            return Err("Shortcut must have 1-3 keys".to_string());
        }
        // Check for duplicates
        let mut seen = HashSet::new();
        for key in &self.keys {
            if !seen.insert(key.keycode) {
                return Err(format!("Duplicate key: {}", key.label));
            }
        }
        Ok(())
    }
}

/// Complete shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutsConfig {
    pub push_to_record: Shortcut,
    pub hands_free_start: Shortcut,
    pub hands_free_stop: Shortcut,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            push_to_record: Shortcut {
                keys: vec![ShortcutKey { keycode: 63, label: "Fn".into() }],
            },
            hands_free_start: Shortcut {
                keys: vec![
                    ShortcutKey { keycode: 63, label: "Fn".into() },
                    ShortcutKey { keycode: 49, label: "Space".into() },
                ],
            },
            hands_free_stop: Shortcut {
                keys: vec![ShortcutKey { keycode: 63, label: "Fn".into() }],
            },
        }
    }
}

impl ConfigKey<ShortcutsConfig> {
    pub const SHORTCUTS: Self = Self::new("shortcutsConfig");
}
```

### Migration Logic

Add to [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs):

```rust
/// Migrate from RecordingTrigger to ShortcutsConfig (run once on startup)
pub fn migrate_trigger_to_shortcuts(store: &impl ConfigStore) -> Result<(), String> {
    // Skip if already migrated
    if store.get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS).is_some() {
        return Ok(());
    }

    // Load old trigger config
    let app_config = store.get(&ConfigKey::APP).unwrap_or_default();
    let trigger_key = match app_config.recording_trigger {
        RecordingTrigger::Fn => ShortcutKey { keycode: 63, label: "Fn".into() },
        RecordingTrigger::Control => ShortcutKey { keycode: 59, label: "Control".into() },
        RecordingTrigger::Option => ShortcutKey { keycode: 58, label: "Option".into() },
        RecordingTrigger::Command => ShortcutKey { keycode: 55, label: "Command".into() },
    };

    // Create new shortcuts config preserving user's trigger choice
    let shortcuts = ShortcutsConfig {
        push_to_record: Shortcut { keys: vec![trigger_key] },
        hands_free_start: Shortcut {
            keys: vec![trigger_key, ShortcutKey { keycode: 49, label: "Space".into() }],
        },
        hands_free_stop: Shortcut { keys: vec![trigger_key] },
    };

    store.set(&ConfigKey::<ShortcutsConfig>::SHORTCUTS, shortcuts)?;
    Ok(())
}
```

### Thread-Safe Hot-Swap State

Create new file: `src-tauri/src/shortcuts_state.rs`

```rust
use crate::config::ShortcutsConfig;
use std::sync::{Arc, RwLock};

/// Thread-safe runtime shortcuts state for hot-swapping
#[derive(Debug, Clone)]
pub struct ShortcutsState {
    inner: Arc<RwLock<ShortcutsConfig>>,
}

impl ShortcutsState {
    pub fn new(config: ShortcutsConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
        }
    }

    /// Read current shortcuts (many readers allowed)
    pub fn get(&self) -> ShortcutsConfig {
        self.inner.read().unwrap().clone()
    }

    /// Update shortcuts (exclusive write access)
    pub fn update(&self, new_config: ShortcutsConfig) -> ShortcutsConfig {
        let mut guard = self.inner.write().unwrap();
        let old = guard.clone();
        *guard = new_config;
        old
    }

    /// Check if any shortcut uses Fn key (for globe key fix)
    pub fn uses_fn_key(&self) -> bool {
        let config = self.get();
        let fn_code = 63u32;

        config.push_to_record.keys.iter().any(|k| k.keycode == fn_code)
            || config.hands_free_start.keys.iter().any(|k| k.keycode == fn_code)
            || config.hands_free_stop.keys.iter().any(|k| k.keycode == fn_code)
    }
}
```

### Updated Keyboard Listener

Modify [keyboard_listener.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs):

**Key Changes:**
1. Replace `recording_trigger: Key` parameter with `shortcuts_state: Arc<ShortcutsState>`
2. Track pressed keys in a `HashSet<u32>` for combination matching
3. Load shortcuts via `shortcuts_state.get()` on each event (hot-swap safe)
4. Match shortcuts using `shortcut.matches(&pressed_keys)`
5. Remove hardcoded LOCK_MODIFIER constant

```rust
use crate::shortcuts_state::ShortcutsState;
use std::collections::HashSet;

pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
}

impl KeyListener {
    pub fn start(
        command_tx: mpsc::Sender<RecordingCommand>,
        state_manager: Arc<RecordingStateManager>,
        shortcuts_state: Arc<ShortcutsState>,
    ) -> Self {
        let thread_handle = thread::spawn(move || {
            let mut pressed_keys: HashSet<u32> = HashSet::new();

            if let Err(err) = grab(move |event| {
                // Get current shortcuts (hot-swap safe)
                let shortcuts = shortcuts_state.get();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let keycode = key.to_macos_keycode();
                        pressed_keys.insert(keycode);

                        // Check shortcuts
                        if shortcuts.push_to_record.matches(&pressed_keys) {
                            let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                            return Some(event); // Pass through (globe key fix handles Fn)
                        }

                        if shortcuts.hands_free_start.matches(&pressed_keys) {
                            let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                            let _ = command_tx.blocking_send(RecordingCommand::LockRecording);
                            // Swallow Space if it's in the combo
                            if shortcuts.hands_free_start.keys.iter().any(|k| k.keycode == 49) {
                                return None;
                            }
                        }

                        Some(event)
                    }
                    EventType::KeyRelease(key) => {
                        let keycode = key.to_macos_keycode();

                        // Check push-to-record BEFORE removing key
                        let was_push_to_record = shortcuts.push_to_record.matches(&pressed_keys);

                        pressed_keys.remove(&keycode);

                        // Release stops recording (unless locked)
                        if was_push_to_record && !state_manager.is_recording_locked() {
                            let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        }

                        // Check hands-free stop
                        if shortcuts.hands_free_stop.matches(&pressed_keys)
                            && state_manager.is_recording_locked() {
                            let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        }

                        Some(event)
                    }
                    _ => Some(event),
                }
            }) {
                error!("Keyboard grab failed: {}", err);
            }
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }
}
```

**Note:** Need to add `to_macos_keycode()` method to `dictara_keyboard::Key` enum in `crates/keyboard/src/key.rs`.

### Backend Commands

Create new file: `src-tauri/src/commands/preferences/shortcuts.rs`

```rust
use crate::config::{ConfigKey, ConfigStore, ShortcutsConfig};
use crate::shortcuts_state::ShortcutsState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn load_shortcuts_config(
    config_store: State<config::Config>,
) -> Result<ShortcutsConfig, String> {
    Ok(config_store.get(&ConfigKey::SHORTCUTS).unwrap_or_default())
}

#[tauri::command]
#[specta::specta]
pub fn save_shortcuts_config(
    config_store: State<config::Config>,
    shortcuts_state: State<ShortcutsState>,
    config: ShortcutsConfig,
) -> Result<(), String> {
    // Validate all shortcuts
    config.push_to_record.validate()?;
    config.hands_free_start.validate()?;
    config.hands_free_stop.validate()?;

    // Save to persistent storage
    config_store.set(&ConfigKey::SHORTCUTS, config.clone())?;

    // Hot-swap runtime state (NO RESTART NEEDED!)
    let old_config = shortcuts_state.update(config.clone());

    // Update globe key fix if Fn usage changed
    let old_uses_fn = old_config.push_to_record.keys.iter().any(|k| k.keycode == 63)
        || old_config.hands_free_start.keys.iter().any(|k| k.keycode == 63);
    let new_uses_fn = shortcuts_state.uses_fn_key();

    if old_uses_fn != new_uses_fn && new_uses_fn {
        crate::globe_key::fix_globe_key_if_needed();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn reset_shortcuts_config(
    config_store: State<config::Config>,
    shortcuts_state: State<ShortcutsState>,
) -> Result<ShortcutsConfig, String> {
    let defaults = ShortcutsConfig::default();
    config_store.set(&ConfigKey::SHORTCUTS, defaults.clone())?;
    shortcuts_state.update(defaults.clone());
    Ok(defaults)
}
```

Register in:
- `src-tauri/src/commands/mod.rs`: add `pub mod shortcuts;`
- `src-tauri/src/commands/registry.rs`: add commands to the list

### App Setup Integration

Modify [setup.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/setup.rs):

```rust
// After line 158 (state_manager creation), add:

// Load or migrate shortcuts config
migrate_trigger_to_shortcuts(&config_store)?;
let shortcuts_config = config_store
    .get(&ConfigKey::<ShortcutsConfig>::SHORTCUTS)
    .unwrap_or_default();

// Create thread-safe shortcuts state
let shortcuts_state = Arc::new(ShortcutsState::new(shortcuts_config.clone()));
app.manage(shortcuts_state.clone());

// Replace lines 204-206 (KeyListener initialization):
if has_accessibility {
    let _listener = KeyListener::start(
        command_tx,
        state_manager.clone(),
        shortcuts_state.clone(), // Pass shortcuts state instead of single trigger
    );
}

// Update lines 216-220 (globe key fix):
if shortcuts_state.uses_fn_key() {
    globe_key::fix_globe_key_if_needed();
}
```

Add to `src-tauri/src/lib.rs`:
```rust
pub mod shortcuts_state;
```

---

## Frontend Implementation

### Key Capture Component

Create `src/components/shortcuts/KeyCaptureInput.tsx`:

**Features:**
- Captures keyboard events when in "capture mode"
- Displays current keys as badges with remove buttons
- Special "Use Fn" button (Fn key can't be captured by JavaScript)
- Max 3 keys enforcement
- Clear all button

**Key Implementation Details:**
```tsx
const handleKeyDown = (e: KeyboardEvent) => {
  e.preventDefault()
  e.stopPropagation()

  const keycode = e.keyCode || e.which
  const label = getKeyLabel(e.key, e)

  const shortcutKey: ShortcutKey = { keycode, label }

  if (value.length < 3 && !value.some(k => k.keycode === keycode)) {
    onChange([...value, shortcutKey])
  }
}

const handleUseFn = () => {
  const fnKey: ShortcutKey = { keycode: 63, label: 'Fn' }
  if (value.length < 3 && !value.some(k => k.keycode === 63)) {
    onChange([...value, fnKey])
  }
}
```

**Key Label Mapping:**
```tsx
const labelMap: Record<string, string> = {
  ' ': 'Space',
  'Control': 'Control',
  'Alt': 'Option',
  'Meta': 'Command',
  'Shift': 'Shift',
  'Enter': 'Return',
  'Escape': 'Esc',
  // Map all common keys
}
```

### Shortcuts Configuration Component

Create `src/components/preferences/ShortcutsConfig.tsx`:

**Features:**
- Uses TanStack Query for data fetching
- Three `KeyCaptureInput` sections (push-to-record, hands-free-start, hands-free-stop)
- Auto-save on changes (no separate save button needed)
- Reset to defaults button
- Info alert: "Changes take effect immediately - no restart required!"

**Query Hooks:**
```tsx
const { data: config } = useQuery({
  queryKey: ['shortcutsConfig'],
  queryFn: () => commands.loadShortcutsConfig()
})

const saveMutation = useMutation({
  mutationFn: (newConfig: ShortcutsConfig) =>
    commands.saveShortcutsConfig(newConfig),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['shortcutsConfig'] })
  }
})
```

### Update Hotkeys Preferences Page

Modify [Hotkeys.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/preferences/Hotkeys.tsx):

```tsx
import { ShortcutsConfiguration } from '@/components/preferences/ShortcutsConfig'

export function Hotkeys() {
  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold">Keyboard Shortcuts</h2>
        <p className="text-sm text-muted-foreground">
          Configure your recording shortcuts
        </p>
      </div>

      <ShortcutsConfiguration />
    </div>
  )
}
```

Remove the old restart logic - hot-swap makes it unnecessary!

### Update Onboarding

**Option 1: Replace TriggerKeyStep** (Recommended)
- Rename/replace [TriggerKeyStep.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/onboarding/steps/TriggerKeyStep.tsx) with `ShortcutsStep.tsx`
- Use `ShortcutsConfiguration` component (same as preferences)
- Update route in `src/routes/onboarding/trigger-key.tsx` → `shortcuts.tsx`
- Update `OnboardingStep` enum in [config.rs:99](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs#L99): change `TriggerKey` to `Shortcuts`
- Update step order in onboarding navigation utils

**Option 2: Keep TriggerKeyStep, Add ShortcutsStep Later**
- Keep existing trigger key step for simplicity
- Users can configure full shortcuts in preferences
- Less disruption to onboarding flow

**Recommendation:** Use Option 1 for consistency with the full vision.

---

## Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/shortcuts_state.rs` | Thread-safe shortcuts runtime state with RwLock |
| `src-tauri/src/commands/preferences/shortcuts.rs` | Load/save/reset shortcuts commands |
| `src/components/shortcuts/KeyCaptureInput.tsx` | Interactive key capture component with Fn button |
| `src/components/preferences/ShortcutsConfig.tsx` | Shortcuts configuration UI (3 sections) |
| `src/components/onboarding/steps/ShortcutsStep.tsx` | Onboarding step for shortcuts (optional, see above) |
| `src/routes/onboarding/shortcuts.tsx` | Route for shortcuts onboarding step |
| `crates/keyboard/src/key.rs` (add method) | Add `to_macos_keycode()` method to Key enum |

---

## Files to Modify

| File | Changes |
|------|---------|
| [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs) | Add ShortcutKey, Shortcut, ShortcutsConfig structs; add migration function |
| [keyboard_listener.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/keyboard_listener.rs) | Replace single trigger with ShortcutsState; track pressed keys HashSet; match combinations |
| [setup.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/setup.rs) | Call migration; create ShortcutsState; pass to KeyListener; update globe key fix |
| [lib.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/lib.rs) | Add `pub mod shortcuts_state;` |
| [commands/mod.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/commands/mod.rs) | Add `pub mod shortcuts;` |
| [commands/registry.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/commands/registry.rs) | Register shortcuts commands |
| [Hotkeys.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/preferences/Hotkeys.tsx) | Replace with ShortcutsConfiguration component; remove restart logic |
| [TriggerKeyStep.tsx](/Users/vitaliizinchenko/Projects/dictara/src/components/onboarding/steps/TriggerKeyStep.tsx) | Rename to ShortcutsStep and use ShortcutsConfiguration |
| `crates/keyboard/src/key.rs` | Add `pub fn to_macos_keycode(&self) -> u32` method |
| [config.rs](/Users/vitaliizinchenko/Projects/dictara/src-tauri/src/config.rs) OnboardingStep | Change `TriggerKey` to `Shortcuts` (or keep both for migration) |

---

## Implementation Phases

### Phase 1: Backend Foundation (No Breaking Changes)
1. Add `ShortcutKey`, `Shortcut`, `ShortcutsConfig` to `config.rs`
2. Add `ConfigKey::SHORTCUTS` constant
3. Add migration function `migrate_trigger_to_shortcuts()`
4. Create `shortcuts_state.rs` with `ShortcutsState`
5. Add `to_macos_keycode()` to `dictara_keyboard::Key`
6. **Keep old RecordingTrigger** for now (backward compatibility)

**Verification:** Run `npm run verify` - should compile with no errors

### Phase 2: Keyboard Listener Update
1. Update `KeyListener::start()` signature to accept `ShortcutsState`
2. Implement pressed keys tracking with `HashSet<u32>`
3. Add shortcut matching logic
4. Remove hardcoded `LOCK_MODIFIER` constant

**Verification:** Test keyboard events still work with simple shortcuts

### Phase 3: Setup Integration & Migration
1. Update `setup.rs` to call migration on startup
2. Create `ShortcutsState` and pass to `KeyListener`
3. Update globe key fix logic
4. Test migration with existing user configs

**Verification:** Existing users' trigger preferences migrate correctly

### Phase 4: Backend Commands
1. Create `commands/preferences/shortcuts.rs`
2. Add `load_shortcuts_config`, `save_shortcuts_config`, `reset_shortcuts_config`
3. Register commands in mod.rs and registry.rs
4. Test hot-swap functionality (save should update without restart)

**Verification:** Commands work via Tauri devtools, hot-swap updates keyboard listener

### Phase 5: Frontend - Key Capture Component
1. Create `KeyCaptureInput.tsx` with keyboard event handling
2. Add "Use Fn" button
3. Implement key label mapping
4. Test capturing various key combinations

**Verification:** Can capture Shift+R, Cmd+Space, etc.; Fn button works

### Phase 6: Frontend - Shortcuts Configuration UI
1. Create `ShortcutsConfig.tsx` with TanStack Query hooks
2. Add three KeyCaptureInput sections
3. Implement auto-save on change
4. Add reset to defaults button

**Verification:** Can configure all 3 shortcuts, changes save and take effect immediately

### Phase 7: Preferences Integration
1. Update `Hotkeys.tsx` to use new component
2. Remove old restart logic and TriggerKeySelector
3. Test in preferences page

**Verification:** Preferences page shows new UI, shortcuts work, no restart needed

### Phase 8: Onboarding Update (Optional)
1. Create `ShortcutsStep.tsx` or update `TriggerKeyStep.tsx`
2. Update routes and navigation
3. Update `OnboardingStep` enum in config.rs
4. Test onboarding flow

**Verification:** Onboarding works with new shortcuts step

### Phase 9: Testing & Edge Cases
1. Test overlapping shortcuts (e.g., Fn vs Fn+Space)
2. Test rapid key presses and releases
3. Test hot-swap while recording
4. Test migration from all old trigger types
5. Test Fn key special handling
6. Run `npm run verify`

**Verification:** All edge cases handled correctly, no crashes

### Phase 10: Cleanup (Optional)
1. Consider removing old `RecordingTrigger` (breaking change)
2. Remove old `TriggerKeySelector` component if unused
3. Update documentation

---

## Key Technical Considerations

### 1. Key Combination Matching Algorithm

**Problem:** How to match exact key combinations without false positives?

**Solution:** HashSet-based exact matching
- Track all pressed keys: `pressed_keys: HashSet<u32>`
- On key press: add to set, check matches
- On key release: check matches FIRST, then remove from set
- Match condition: `shortcut.keys.len() == pressed_keys.len() && all_present`

**Why this works:**
- Prevents subset matches (Fn won't match Fn+Space)
- Order-independent (Shift+R == R+Shift)
- O(1) lookup with HashSet
- Race condition safe (check before removing)

### 2. Hot-Swap Mechanism

**Problem:** Keyboard listener runs in separate thread, how to update without restart?

**Solution:** RwLock-based shared state
- `ShortcutsState` wraps config in `Arc<RwLock<ShortcutsConfig>>`
- Keyboard listener calls `shortcuts_state.get()` on each event
- Save command calls `shortcuts_state.update(new_config)`
- RwLock allows many concurrent readers (fast path)
- Write lock only needed during config updates (rare)

**Performance:** ~10ns overhead per event (negligible)

### 3. Fn Key Special Handling

**Problem:** JavaScript cannot capture Fn key in webview

**Solutions:**
- Frontend: "Use Fn" button that directly adds `{ keycode: 63, label: "Fn" }`
- Backend: Fn key (keycode 63) works normally in Rust keyboard listener
- UI: Show Fn badge clearly when it's part of a shortcut

### 4. Space Key Swallowing

**Current behavior:** Space is swallowed when it locks recording

**New behavior:** Only swallow Space if it's part of `hands_free_start` shortcut
- Check: `hands_free_start.keys.iter().any(|k| k.keycode == 49)`
- Allows Space in other contexts (e.g., Ctrl+Space for trigger)

### 5. Migration Strategy

**Non-destructive migration:**
1. Check if `shortcutsConfig` exists → skip migration
2. Load old `recordingTrigger` from `appConfig`
3. Convert to new 3-shortcut format (preserving user's trigger choice)
4. Save to `shortcutsConfig`
5. **Keep old config** for rollback safety

**Backward compatibility:** Default shortcuts match old behavior exactly

### 6. Globe Key Fix Dynamic Update

**Current:** Globe key fix applied once at startup if using Fn

**New:** Update dynamically when shortcuts change
- Track old vs new Fn usage in `save_shortcuts_config`
- Only call `fix_globe_key_if_needed()` when transitioning from no-Fn to Fn
- Leave setting unchanged when removing Fn (safer UX)

### 7. Validation

**Backend validation (critical):**
- 1-3 keys per shortcut
- No duplicate keys within a shortcut
- Valid keycodes (0-127 range)
- Return clear error messages

**Frontend validation (UX):**
- Disable capture button after 3 keys
- Show error toast on save failure
- Prevent clearing required shortcuts

### 8. Thread Safety

**Why RwLock over Mutex:**
- Keyboard events: ~50-100/sec (many reads)
- Config updates: ~1/hour (rare writes)
- RwLock allows parallel reads → better performance

**Clone strategy:**
- `ShortcutsConfig` is small (~100 bytes)
- Cloning is cheaper than holding read lock across event processing
- Prevents lock contention and potential deadlocks

---

## Edge Cases & Mitigations

| Edge Case | Mitigation |
|-----------|------------|
| Overlapping shortcuts (Fn vs Fn+Space) | Exact length matching prevents subset matches |
| Same shortcut for multiple actions | Backend validation warns about conflicts |
| Fn key can't be captured by JS | Special "Use Fn" button in UI |
| User holds 4+ keys | Frontend limits to 3 keys max |
| Rapid key presses | HashSet ensures correct state tracking |
| Hot-swap during recording | State manager handles gracefully, recording continues |
| Migration fails | Config falls back to defaults (Fn trigger) |
| Invalid keycodes from frontend | Backend validation rejects and returns error |
| Thread deadlock | RwLock + clone strategy prevents holding locks |
| Globe key fix race condition | Only update on Fn usage change, not every save |

---

## Verification & Testing

### Manual Testing Checklist

**Backend:**
- [ ] Migration from all 4 old trigger types (Fn, Control, Option, Command)
- [ ] Hot-swap: change shortcuts without restart, verify they work immediately
- [ ] Validation: try saving invalid shortcuts (0 keys, 4 keys, duplicates)
- [ ] Globe key fix: verify updates when Fn usage changes

**Frontend:**
- [ ] Key capture: capture Shift+R, Cmd+Space, Control+Option+F
- [ ] Fn button: add Fn key to shortcuts
- [ ] Max keys: verify cannot add 4th key
- [ ] Remove keys: click X button to remove individual keys
- [ ] Clear: clear all keys and re-capture
- [ ] Reset: reset to defaults button works

**Integration:**
- [ ] Push-to-record: hold keys, release stops recording
- [ ] Hands-free start: press combo, recording locks
- [ ] Hands-free stop: press combo, recording stops
- [ ] Overlapping shortcuts: Fn and Fn+Space work correctly (no false triggers)
- [ ] Space swallowing: Space only swallowed in hands-free-start combo
- [ ] Onboarding: new shortcuts step works, saves config

**Edge Cases:**
- [ ] Press keys in different orders (Shift+R vs R+Shift)
- [ ] Rapid press/release cycles
- [ ] Change shortcuts while recording (should handle gracefully)
- [ ] Migrate from old config, verify preserved trigger choice
- [ ] Multiple users with different configs (no cross-contamination)

### Automated Tests

**Backend (Rust):**
```rust
#[test]
fn test_shortcut_matching() {
    let shortcut = Shortcut {
        keys: vec![
            ShortcutKey { keycode: 56, label: "Shift" },
            ShortcutKey { keycode: 17, label: "R" },
        ],
    };

    let mut pressed = HashSet::new();
    pressed.insert(56);
    pressed.insert(17);
    assert!(shortcut.matches(&pressed));

    // Subset should not match
    pressed.remove(&17);
    assert!(!shortcut.matches(&pressed));
}

#[test]
fn test_migration() {
    let store = MockConfigStore::new();

    // Set old trigger
    store.set(&ConfigKey::APP, AppConfig {
        recording_trigger: RecordingTrigger::Control,
        ..Default::default()
    });

    // Run migration
    migrate_trigger_to_shortcuts(&store).unwrap();

    // Verify new config
    let shortcuts = store.get(&ConfigKey::SHORTCUTS).unwrap();
    assert_eq!(shortcuts.push_to_record.keys[0].keycode, 59); // Control
}
```

**Frontend (TypeScript/Vitest):**
```tsx
test('KeyCaptureInput captures keys', async () => {
  const onChange = vi.fn()
  render(<KeyCaptureInput value={[]} onChange={onChange} />)

  fireEvent.click(screen.getByText('Capture Keys'))
  fireEvent.keyDown(window, { key: 'Shift', keyCode: 56 })

  expect(onChange).toHaveBeenCalledWith([
    { keycode: 56, label: 'Shift' }
  ])
})
```

### Build Verification

Run after each phase:
```bash
npm run verify
```

This runs:
- Rust compilation and tests
- TypeScript type checking
- Tauri-specta binding generation
- Frontend build

All must pass before proceeding to next phase.

---

## Success Criteria

- [ ] Existing users seamlessly migrated from simple trigger to 3-shortcut system
- [ ] All three shortcuts configurable with 1-3 key combinations
- [ ] Changes take effect immediately without app restart
- [ ] Key capture UI works intuitively (including Fn button)
- [ ] No regressions in recording functionality
- [ ] `npm run verify` passes completely
- [ ] Manual testing checklist 100% complete
- [ ] Code follows existing patterns (ConfigStore, tauri-specta, TanStack Query)

---

## Rollback Plan

If critical issues are discovered:

1. **Phase 1-3 (Backend only):** Revert commits, migration is backward compatible
2. **Phase 4-7 (Commands + Frontend):** Keep backend, disable frontend UI, fall back to simple trigger selector
3. **Phase 8+ (Onboarding):** Keep old TriggerKeyStep, don't activate ShortcutsStep

Migration is non-destructive - old `appConfig.recordingTrigger` is preserved.

---

## Future Enhancements (Out of Scope)

- Dynamic shortcut labels in onboarding tutorials (FnHoldStep, FnSpaceStep)
- Conflict detection warning when shortcuts overlap
- Export/import shortcuts config
- Shortcut presets (Vim mode, Emacs mode, etc.)
- Global keyboard listener (work in any app, not just when focused)
- Visual keyboard layout for configuration
- Accessibility: screen reader support for key capture

These can be added incrementally after the core feature is stable.
