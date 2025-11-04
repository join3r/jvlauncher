# macOS Design Standards - Quick Reference

This document provides a quick reference for the macOS design standards implemented in jvlauncher.

## Typography

### Font Sizes
```css
/* Standard sizes */
--font-size-title: 13px;        /* Window/section titles */
--font-size-body: 13px;         /* Body text, labels, buttons */
--font-size-small: 11px;        /* Secondary text, hints, captions */

/* Font weights */
--font-weight-regular: 400;     /* Default for most text */
--font-weight-medium: 500;      /* Rarely used */
--font-weight-semibold: 600;    /* Titles only */
```

### Font Family
```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
```

## Controls

### Buttons
```css
/* Standard button */
padding: 4px 14px;
font-size: 13px;
font-weight: 400;
border-radius: 6px;
border: 0.5px solid var(--border-color);
height: 24px;
line-height: 1.2;

/* Primary button (blue) */
background: #007aff;
color: white;
border-color: #007aff;

/* Secondary button */
background: var(--bg-secondary);
color: var(--text-primary);
border: 0.5px solid var(--border-color);
```

### Text Inputs
```css
padding: 3px 6px;
font-size: 13px;
border-radius: 5px;
border: 0.5px solid var(--border-color);
height: 22px;
line-height: 1.2;
background: var(--bg-tertiary);
```

### Segmented Controls
```css
/* Container */
padding: 2px;
background: var(--bg-tertiary);
border-radius: 6px;
border: 0.5px solid var(--border-color);

/* Buttons */
padding: 3px 12px;
font-size: 13px;
font-weight: 400;
border-radius: 4px;

/* Active state */
background: var(--accent);
color: #fff;
box-shadow: 0 1px 2px rgba(0,0,0,0.15);
```

### Checkboxes
```css
/* Use native size - no scaling */
accent-color: var(--accent);
```

## Spacing

### Container Padding
```css
/* Settings/form containers */
padding: 16px 20px;

/* Modals */
padding: 16px 20px;  /* macOS */
padding: 20px;       /* Generic */
```

### Margins and Gaps
```css
/* Form groups */
margin-bottom: 12px;

/* Between elements in a row */
gap: 6px;  /* Tight spacing (buttons, inputs) */
gap: 8px;  /* Standard spacing */
gap: 12px; /* Loose spacing (sections) */

/* Grid layouts */
row-gap: 12px;
column-gap: 12px;
```

### Title Margins
```css
/* Section titles */
margin-top: 0;
margin-bottom: 16px;
```

## Borders and Radii

### Border Widths
```css
/* Standard border */
border: 0.5px solid var(--border-color);

/* No border for some macOS elements */
border: none;
```

### Border Radius
```css
/* Buttons, inputs */
border-radius: 5px;  /* Small controls */
border-radius: 6px;  /* Standard buttons */

/* Containers */
border-radius: 10px; /* Small containers */
border-radius: 12px; /* Modals, cards */
border-radius: 14px; /* Large containers (macOS) */
```

## Shadows

### Button Shadows
```css
/* Default */
box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);

/* Primary button */
box-shadow: 0 1px 2px rgba(0, 122, 255, 0.2);

/* Hover state */
box-shadow: 0 1px 3px rgba(0, 122, 255, 0.3);
```

### Focus States
```css
/* Standard focus ring */
outline: none;
border-color: var(--accent);
box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);

/* macOS specific */
box-shadow: 
    0 0 0 3px rgba(0, 122, 255, 0.15),
    0 1px 2px rgba(0, 0, 0, 0.05);
```

## Colors

### macOS Light Mode
```css
--accent: #007aff;                      /* System blue */
--bg-primary: rgba(255, 255, 255, 0.50);
--bg-secondary: rgba(255, 255, 255, 0.60);
--bg-tertiary: rgba(255, 255, 255, 0.40);
--text-primary: #1d1d1f;
--text-secondary: #6e6e73;
--border-color: rgba(0, 0, 0, 0.04);
```

### macOS Dark Mode
```css
--accent: #0a84ff;
--bg-primary: rgba(36, 36, 38, 0.60);
--bg-secondary: rgba(46, 46, 50, 0.70);
--bg-tertiary: rgba(30, 30, 34, 0.55);
--text-primary: #f5f5f7;
--text-secondary: #c7c7cb;
--border-color: rgba(255, 255, 255, 0.08);
```

## Layout Patterns

### Settings Row Layout (Settings Window)
```css
/* Grid-based two-column layout */
display: grid;
grid-template-columns: 140px 1fr;
row-gap: 12px;
column-gap: 12px;
align-items: center;

/* Labels */
text-align: right;
font-weight: 400;
font-size: 13px;

/* Controls */
display: flex;
gap: 8px;
align-items: center;
```

### Form Layout (Add/Edit Application)
```css
/* Two-column grid layout (macOS native pattern) */
.form-rows {
    display: grid;
    grid-template-columns: 140px 1fr;
    row-gap: 12px;
    column-gap: 12px;
    align-items: start;
}

/* Labels - right-aligned in left column */
.form-label {
    font-weight: 400;
    font-size: 13px;
    text-align: right;
    padding-top: 3px; /* Align with input text */
}

/* Controls - in right column */
.form-control {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
}
```

### Legacy Vertical Form Layout
```css
/* Vertical form groups (for backwards compatibility) */
.form-group {
    margin-bottom: 12px;
}

.form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 400;
    font-size: 13px;
}
```

### Button Groups
```css
/* Horizontal button group */
display: flex;
gap: 8px;
justify-content: flex-end;
margin-top: 16px;
```

## Transitions

### Standard Transitions
```css
/* Most interactive elements */
transition: all 0.15s ease;

/* Specific properties */
transition: background 0.15s, color 0.15s;
```

## Comparison with Native macOS Apps

### Mail.app Settings
- Font size: 13px ✓
- Button height: ~24px ✓
- Input height: ~22px ✓
- Label weight: Regular (400) ✓
- Spacing: Compact ✓

### System Preferences
- Font size: 13px ✓
- Segmented controls: Compact ✓
- Button padding: Minimal ✓
- Border radius: 5-6px ✓

## Implementation Notes

1. **Always use fixed heights** for buttons and inputs (24px and 22px respectively)
2. **Use line-height: 1.2** to ensure proper vertical centering
3. **Prefer regular font weight (400)** over medium/semibold
4. **Use 0.5px borders** for a lighter, more refined appearance
5. **Keep spacing tight** - 6-8px between related elements, 12px between groups
6. **Use system colors** via CSS variables for automatic dark mode support

## Testing Checklist

- [ ] Font sizes are 13px for body text, 11px for small text
- [ ] Buttons are 24px tall with 4px vertical padding
- [ ] Inputs are 22px tall with 3px vertical padding
- [ ] Spacing between form groups is 12px
- [ ] Labels use regular (400) font weight
- [ ] Border radius is 5-6px for controls
- [ ] Focus states use 20% opacity accent color
- [ ] All text is readable in both light and dark modes
- [ ] Layout matches native macOS application density

## Resources

- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/macos)
- [macOS Design Resources](https://developer.apple.com/design/resources/)
- [SF Symbols](https://developer.apple.com/sf-symbols/)

