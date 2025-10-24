# Multi-Monitor Testing Checklist

**Date**: 2025-01-25  
**Branch**: m5-multi-monitor  
**Hardware**: eDP-1 (2160x1440) + HDMI-A-1 (2560x1440@144Hz)

## Test Environment

### Display Configuration
```
Monitor 1: eDP-1 (Laptop)
- Resolution: 2160x1440@60Hz
- Description: BOE 0x0893

Monitor 2: HDMI-A-1 (External)
- Resolution: 2560x1440@144Hz
- Description: HKC G24H2
```

### Test Videos Created
- ✅ `laptop-specific.mp4` - RED (440Hz tone)
- ✅ `hdmi-a-pattern.mp4` - GREEN (880Hz tone)
- ✅ `hdmi-generic.mp4` - BLUE (220Hz tone)
- ✅ `fallback.mp4` - YELLOW (330Hz tone)
- ✅ `default-test.mp4` - WHITE (550Hz tone)

All videos: 10 seconds, 1920x1080, ~102KB each

## Test Plan

### Test 1: Pattern Matching ⏳
**Objective**: Verify pattern matching and priority work correctly

**Expected**:
- [ ] eDP-1 shows RED video (exact match, priority 0)
- [ ] HDMI-A-1 shows GREEN video (HDMI-A-* pattern, priority 5)
- [ ] Both play smoothly at 30 FPS (per config)

**Config rules tested**:
```yaml
"eDP-1": exact match (priority 0 implicit)
"HDMI-A-*": pattern priority 5
"HDMI-*": pattern priority 10 (should NOT match)
"*": fallback priority 99 (should NOT match)
```

**Result**: _______

---

### Test 2: IPC Status Command ⏳
**Objective**: Verify status reporting works

**Expected**:
- [ ] JSON output with 2 outputs
- [ ] Each output shows correct source path
- [ ] Status includes resolution, fps, etc.

**Result**: _______

---

### Test 3: Dynamic Source Switching (File) ⏳
**Objective**: Verify runtime source switching with file://

**Actions**:
```bash
wayvid-ctl switch -o HDMI-A-1 /home/yangyus8/Videos/hdmi-generic.mp4
```

**Expected**:
- [ ] HDMI-A-1 changes from GREEN to BLUE
- [ ] Transition is smooth (no crash)
- [ ] eDP-1 unaffected (still RED)

**Result**: _______

---

### Test 4: Dynamic Source Switching (Another File) ⏳
**Objective**: Verify switching laptop display

**Actions**:
```bash
wayvid-ctl switch -o eDP-1 /home/yangyus8/Videos/fallback.mp4
```

**Expected**:
- [ ] eDP-1 changes from RED to YELLOW
- [ ] HDMI-A-1 unaffected (still BLUE)
- [ ] No memory leaks or stuttering

**Result**: _______

---

### Test 5: Config Reload ⏳
**Objective**: Verify config reload restores original sources

**Actions**:
```bash
wayvid-ctl reload
```

**Expected**:
- [ ] eDP-1 back to RED (laptop-specific.mp4)
- [ ] HDMI-A-1 back to GREEN (hdmi-a-pattern.mp4)
- [ ] Pattern matching re-applied correctly

**Result**: _______

---

### Test 6: Pause/Resume Control ⏳
**Objective**: Verify per-output playback control

**Actions**:
```bash
wayvid-ctl pause -o HDMI-A-1
wayvid-ctl resume -o HDMI-A-1
```

**Expected**:
- [ ] HDMI-A-1 pauses (frame frozen)
- [ ] eDP-1 continues playing
- [ ] Resume restores playback

**Result**: _______

---

### Test 7: Volume Control ⏳
**Objective**: Verify per-output volume control

**Actions**:
```bash
wayvid-ctl volume -o eDP-1 0.8
wayvid-ctl volume -o HDMI-A-1 0.2
```

**Expected**:
- [ ] Commands execute without error
- [ ] Volume changes audible (if audio enabled)

**Result**: _______

---

### Test 8: Hot-Plug Handling ⏳
**Objective**: Verify monitor disconnect/reconnect behavior

**Actions**:
1. Disconnect HDMI-A-1
2. Wait 3 seconds
3. Reconnect HDMI-A-1

**Expected**:
- [ ] wayvid doesn't crash on disconnect
- [ ] On reconnect, HDMI-A-1 shows GREEN (HDMI-A-* pattern)
- [ ] Pattern matching re-applies correctly

**Result**: _______

---

## Performance Tests

### Memory Usage ⏳
**Expected**: < 512MB total (per config)

**Command**:
```bash
ps aux | grep wayvid | grep -v grep
```

**Result**: _______

---

### CPU Usage ⏳
**Expected**: < 10% per output under normal playback

**Command**:
```bash
top -b -n 1 | grep wayvid
```

**Result**: _______

---

### Frame Timing ⏳
**Expected**: Adaptive frame skipping works

**Check**:
- [ ] No stuttering during normal load
- [ ] Graceful degradation under high load
- [ ] Frame timing logs show proper behavior

**Result**: _______

---

## Edge Cases

### Invalid Pattern ⏳
**Test**: Add invalid pattern in config

**Expected**:
- [ ] Fallback to `*` pattern
- [ ] No crash

**Result**: _______

---

### Non-existent Source ⏳
**Test**: Switch to non-existent file

**Expected**:
- [ ] Error reported
- [ ] Output continues with previous source
- [ ] No crash

**Result**: _______

---

## Final Checklist

- [ ] All 8 main tests pass
- [ ] Performance within acceptable range
- [ ] No crashes or memory leaks
- [ ] Logs show correct pattern matching
- [ ] Documentation is accurate

## Sign-off

**Tested by**: _______  
**Date**: _______  
**Overall Result**: ⏳ PENDING / ✅ PASS / ❌ FAIL

**Notes**:
_______________________________
_______________________________
_______________________________

## Next Steps

If all tests pass:
1. ✅ Commit test results
2. ✅ Create PR #21
3. ✅ Merge to main
4. ✅ Close Issue #2

If issues found:
1. Document issues
2. Fix bugs
3. Re-test
