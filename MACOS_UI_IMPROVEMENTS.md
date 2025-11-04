# macOS UI Improvements - Native Design Standards

## Overview
This document details the comprehensive UI improvements made to jvlauncher to match native macOS application standards according to the macOS Human Interface Guidelines.

## Changes Summary

### 1. Font Size Adjustments

#### Before:
- Labels: 14-15px
- Body text: 14px
- Buttons: 14px
- Small text: 13px

#### After (macOS Native):
- Labels: 13px (regular weight, not bold)
- Body text: 13px
- Buttons: 13px
- Small text/hints: 11px
- Titles: 13px (600 weight)

**Rationale**: Native macOS applications use 13px as the standard system font size for most UI elements, with 11px for secondary information. This creates a more compact, professional appearance.

### 2. Button Styling

#### Before:
- Padding: 8-10px vertical, 18-20px horizontal
- Font size: 14px
- Font weight: 500-600
- Border radius: 6-10px
- Height: Variable (auto)

#### After (macOS Native):
- Padding: 4px vertical, 14px horizontal
- Font size: 13px
- Font weight: 400 (regular)
- Border radius: 6px
- Height: 24px (fixed)
- Line height: 1.2

**Rationale**: macOS native buttons are more compact with less padding and regular font weight, creating a cleaner, less "heavy" appearance.

### 3. Input Field Styling

#### Before:
- Padding: 8-10px vertical, 10-12px horizontal
- Font size: 14px
- Border radius: 6-8px
- Height: Variable (auto)

#### After (macOS Native):
- Padding: 3px vertical, 6px horizontal
- Font size: 13px
- Border radius: 5px
- Height: 22px (fixed)
- Line height: 1.2
- Border: 0.5px (thinner)

**Rationale**: Native macOS text fields are significantly more compact, with minimal padding and a fixed height that matches system standards.

### 4. Spacing and Layout

#### Before:
- Form group margin: 14-20px
- Container padding: 18-30px
- Gap between elements: 12px
- Row gap in settings: 10px

#### After (macOS Native):
- Form group margin: 12px
- Container padding: 16-20px
- Gap between elements: 6-8px
- Row gap in settings: 12px
- Modal padding: 16-20px

**Rationale**: macOS applications use tighter, more consistent spacing that creates a denser, more information-rich interface without feeling cramped.

### 5. Segmented Controls

#### Before:
- Padding: 3px container, 6px buttons
- Button padding: 6px vertical, 14px horizontal
- Font size: 14px
- Border radius: 10px container, 8px buttons

#### After (macOS Native):
- Padding: 2px container, 3px buttons
- Button padding: 3px vertical, 12px horizontal
- Font size: 13px
- Border radius: 6px container, 4px buttons
- Font weight: 400 (regular)

**Rationale**: Native macOS segmented controls are more compact with tighter spacing and smaller border radii.

### 6. Checkbox Styling

#### Before:
- Transform: scale(1.25) - artificially enlarged
- Custom accent color

#### After (macOS Native):
- Native size (no scaling)
- System accent color

**Rationale**: macOS checkboxes should use their native size and appearance for consistency with system UI.

### 7. Modal and Container Styling

#### Before:
- Modal padding: 24-30px
- Border radius: 12px
- Title font size: 20-22px
- Title margin bottom: 20px

#### After (macOS Native):
- Modal padding: 16-20px
- Border radius: 10-12px
- Title font size: 13px
- Title margin bottom: 16px
- Title font weight: 600

**Rationale**: Smaller, more compact modals with consistent typography match native macOS preferences windows.

### 8. Record Button (Shortcut Recording)

#### Before:
- Padding: 8-10px vertical, 14-16px horizontal
- Font size: 14px
- Font weight: 500
- Border radius: 6-8px

#### After (macOS Native):
- Padding: 3px vertical, 12px horizontal
- Font size: 13px
- Font weight: 400
- Border radius: 5px
- Height: 22px (matches input fields)

**Rationale**: Record buttons should match the height and styling of adjacent input fields for visual consistency.

### 9. Icon Preview and Tips

#### Before:
- Icon preview size: 48px
- Tip font size: 11px
- Margins: 8px

#### After (macOS Native):
- Icon preview size: 40px
- Tip font size: 11px
- Margins: 6px
- Reduced opacity: 0.75

**Rationale**: Smaller icons and tighter spacing create a more compact form layout.

### 10. Focus States

#### Before:
- Focus ring: 3px with 25% opacity
- Box shadow: 0 0 0 3px

#### After (macOS Native):
- Focus ring: 3px with 20% opacity
- Box shadow: 0 0 0 3px (slightly more subtle)

**Rationale**: Slightly more subtle focus states match macOS system behavior.

### 11. Form Layout (Add/Edit Application)

#### Before:
- Vertical layout with labels above inputs
- Each field stacked vertically
- Labels left-aligned

#### After (macOS Native):
- Horizontal two-column grid layout
- Labels in left column (140px wide), right-aligned
- Controls in right column
- 12px row gap, 12px column gap
- Labels aligned with input text (3px padding-top)

**Rationale**: Native macOS preference windows use horizontal label-input layouts with right-aligned labels, creating a cleaner, more organized appearance that matches System Preferences and other native apps.

## Files Modified

1. **dist/settings.html**
   - Updated all inline styles for font sizes, spacing, and button dimensions
   - Adjusted grid input widths and labels
   - Updated update section styling

2. **dist/app-form.html**
   - **Changed from vertical to horizontal two-column grid layout**
   - Labels now positioned to the left of inputs (right-aligned)
   - Controls positioned in the right column
   - Updated form group styling
   - Adjusted input field dimensions
   - Updated button and segmented control styling
   - Refined icon preview and tip styling

3. **dist/app-form.js**
   - Updated field visibility logic to handle both labels and controls
   - Added support for showing/hiding label elements separately

4. **dist/styles.css**
   - Updated global button styles (.btn)
   - Refined macOS-specific button variants
   - Updated form group and input styling
   - Adjusted modal content padding and spacing
   - Updated record button styling
   - Refined focus states

## Visual Comparison

### Key Improvements:
1. **More Compact**: Reduced vertical spacing creates a denser, more professional layout
2. **Consistent Typography**: 13px standard font size throughout matches macOS system fonts
3. **Lighter Weight**: Regular (400) font weight instead of medium/semibold creates a cleaner look
4. **Tighter Controls**: Smaller buttons and inputs match native macOS control dimensions
5. **Better Hierarchy**: Consistent spacing and sizing creates clearer visual hierarchy

## Testing

The application has been built and tested with these changes. All UI elements now closely match the appearance of native macOS applications like Mail, System Preferences, and other first-party Apple applications.

## Compatibility

These changes are specifically optimized for macOS. The existing platform detection system ensures that:
- macOS users see the native-styled interface
- Windows/Linux users continue to see Material Design styling
- All functionality remains unchanged

## Future Considerations

1. Consider adding system font stack for even better native appearance
2. Explore using native macOS controls via Tauri plugins for perfect system integration
3. Test with different macOS versions (Big Sur, Monterey, Ventura, Sonoma) for consistency
4. Consider adding support for macOS accent color preferences

## Conclusion

These changes bring jvlauncher's UI in line with macOS Human Interface Guidelines, creating a more professional, native-feeling application that seamlessly integrates with the macOS ecosystem.

