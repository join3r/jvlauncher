# Form Layout Changes - Horizontal Two-Column Grid

## Overview

The Add/Edit Application form has been updated from a vertical label-above-input layout to a horizontal two-column grid layout matching native macOS application design patterns.

## Visual Comparison

### Before (Vertical Layout)
```
┌─────────────────────────────────────────┐
│  Type                                   │
│  [Application] [Web App] [Terminal]     │
│                                         │
│  Name                                   │
│  [Application name____________]         │
│                                         │
│  Binary Path                            │
│  [/path/to/binary_________] [Browse]    │
│                                         │
│  Command Line Parameters                │
│  [--flag value_____________]            │
│                                         │
│  Icon                                   │
│  [Choose Icon] [Paste Icon]             │
│                                         │
│  Keyboard Shortcut                      │
│  [CommandOrControl+1______] [Record]    │
│                                         │
│              [Cancel] [Save]            │
└─────────────────────────────────────────┘
```

### After (Horizontal Two-Column Grid - macOS Native)
```
┌─────────────────────────────────────────┐
│              Type: [App] [Web] [Term]   │
│              Name: [Application name__] │
│       Binary Path: [/path/to/binary__] [Browse] │
│    CLI Parameters: [--flag value_____] │
│              Icon: [Choose] [Paste]     │
│          Shortcut: [Cmd+1___] [Record]  │
│                                         │
│              [Cancel] [Save]            │
└─────────────────────────────────────────┘
```

## Key Changes

### 1. Layout Structure

**Before:**
- Vertical stacking of form groups
- Labels positioned above inputs
- Each field takes full width

**After:**
- Two-column grid layout
- Labels in left column (140px wide)
- Controls in right column (flexible width)
- More compact, information-dense layout

### 2. Label Styling

**Before:**
```css
.form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 400;
    font-size: 13px;
}
```

**After:**
```css
.form-label {
    font-weight: 400;
    font-size: 13px;
    text-align: right;
    padding-top: 3px; /* Align with input text */
}
```

### 3. Grid Configuration

```css
.form-rows {
    display: grid;
    grid-template-columns: 140px 1fr;
    row-gap: 12px;
    column-gap: 12px;
    align-items: start;
}
```

### 4. Control Container

```css
.form-control {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
}
```

## HTML Structure Changes

### Before (Vertical)
```html
<div class="form-group">
    <label>Name</label>
    <input type="text" id="app-name" placeholder="Application name">
</div>

<div class="form-group">
    <label>Binary Path</label>
    <div class="input-with-button">
        <input type="text" id="app-binary" placeholder="/path/to/binary">
        <button class="btn btn-primary">Browse</button>
    </div>
</div>
```

### After (Horizontal Grid)
```html
<div class="form-rows">
    <div class="form-label">Name</div>
    <div class="form-control">
        <input type="text" id="app-name" placeholder="Application name">
    </div>
    
    <div class="form-label">Binary Path</div>
    <div class="form-control">
        <input type="text" id="app-binary" placeholder="/path/to/binary" style="flex: 1;">
        <button class="btn btn-primary">Browse</button>
    </div>
</div>
```

## JavaScript Changes

### Field Visibility Logic

**Before:**
```javascript
function updateFieldsVisibility() {
    const type = getTypeFromSegment();
    document.getElementById('url-group').style.display = type === 'webapp' ? 'block' : 'none';
    document.getElementById('binary-group').style.display = type !== 'webapp' ? 'block' : 'none';
    document.getElementById('params-group').style.display = type !== 'webapp' ? 'block' : 'none';
}
```

**After:**
```javascript
function updateFieldsVisibility() {
    const type = getTypeFromSegment();
    // Show/hide both label and control for each field
    const urlLabel = document.getElementById('url-label');
    const urlGroup = document.getElementById('url-group');
    const binaryLabel = document.getElementById('binary-label');
    const binaryGroup = document.getElementById('binary-group');
    const paramsLabel = document.getElementById('params-label');
    const paramsGroup = document.getElementById('params-group');
    
    if (type === 'webapp') {
        urlLabel.style.display = 'block';
        urlGroup.style.display = 'flex';
        binaryLabel.style.display = 'none';
        binaryGroup.style.display = 'none';
        paramsLabel.style.display = 'none';
        paramsGroup.style.display = 'none';
    } else {
        urlLabel.style.display = 'none';
        urlGroup.style.display = 'none';
        binaryLabel.style.display = 'block';
        binaryGroup.style.display = 'flex';
        paramsLabel.style.display = 'block';
        paramsGroup.style.display = 'flex';
    }
}
```

## Benefits

### 1. Native macOS Appearance
- Matches System Preferences and other native macOS applications
- Follows Apple Human Interface Guidelines
- Creates a more professional, polished look

### 2. Better Space Utilization
- More compact layout reduces vertical scrolling
- Better use of horizontal space
- Clearer visual hierarchy

### 3. Improved Scannability
- Right-aligned labels create a clear visual column
- Easier to scan and find specific fields
- Better alignment of related elements

### 4. Consistency
- Matches the settings.html layout pattern
- Consistent design language throughout the application
- Predictable user experience

## Comparison with Native macOS Apps

### Mail.app Account Settings
- ✅ Two-column grid layout
- ✅ Right-aligned labels
- ✅ 140px label column width
- ✅ Compact spacing (12px gaps)
- ✅ Labels aligned with input text

### System Preferences
- ✅ Horizontal label-control pairs
- ✅ Right-aligned labels
- ✅ Consistent column widths
- ✅ Minimal vertical spacing

## Implementation Details

### Label Width
- **140px** - Chosen to accommodate the longest label ("CLI Parameters") while maintaining compact layout
- Consistent with settings.html for visual harmony

### Alignment
- **Labels**: Right-aligned with 3px top padding to align with input text baseline
- **Controls**: Left-aligned in right column with flex layout for multi-element rows

### Spacing
- **Row gap**: 12px - Matches macOS native spacing
- **Column gap**: 12px - Creates clear separation between labels and controls
- **Internal control gap**: 6px - Tight spacing for buttons within a row

### Responsive Behavior
- Labels maintain fixed 140px width
- Controls flex to fill available space
- Multi-element controls wrap if needed (flex-wrap: wrap)

## Testing Checklist

- [x] Layout matches native macOS application patterns
- [x] Labels are right-aligned in left column
- [x] Controls are properly aligned in right column
- [x] Field visibility toggling works correctly (webapp vs app vs tui)
- [x] Both labels and controls show/hide together
- [x] Spacing is consistent (12px gaps)
- [x] Input fields with buttons (Browse, Record) display correctly
- [x] Icon section with multiple buttons and preview displays correctly
- [x] Form is readable in both light and dark modes
- [x] No layout breaking or overflow issues

## Files Modified

1. **dist/app-form.html**
   - Changed from `.form-group` vertical layout to `.form-rows` grid layout
   - Split labels into separate `.form-label` divs
   - Wrapped controls in `.form-control` divs
   - Added IDs to labels for visibility toggling
   - Updated container max-width to 580px (from 520px) to accommodate wider layout

2. **dist/app-form.js**
   - Updated `updateFieldsVisibility()` function
   - Added logic to show/hide both labels and controls
   - Changed display values from 'block' to 'flex' for controls

3. **dist/app-form.html (CSS)**
   - Added `.form-rows` grid layout styles
   - Added `.form-label` right-aligned label styles
   - Added `.form-control` flex container styles
   - Updated input/select styles to work with new layout
   - Maintained backwards compatibility with `.form-group` for other uses

## Migration Notes

### For Future Forms
When creating new forms in jvlauncher, use the horizontal two-column grid pattern:

```html
<div class="form-rows">
    <div class="form-label">Label Text</div>
    <div class="form-control">
        <!-- Input or other controls -->
    </div>
</div>
```

### Backwards Compatibility
The old `.form-group` vertical layout styles are still available for any legacy code or special cases where vertical layout is preferred.

## Conclusion

This change brings the Add/Edit Application form in line with native macOS design patterns, creating a more professional, compact, and scannable interface that matches user expectations for macOS applications.

