# Make Mycast yours

Search for **Change Theme**, or open Preferences → Appearance. Choose Mycast Dark, Midnight, Graphite, Paper, or Forest. Theme selection is saved on this device.

Choose **Manual** to keep one theme, or **Follow Windows** to select separate dark and light themes. Each selector includes matching built-in and custom themes; the defaults are Mycast Dark and Paper. Windows color-scheme changes update the application through an event listener. Selecting or saving a theme directly switches to Manual. Deleting a custom theme used in the system pair restores its corresponding default. Editor previews remain temporary; Cancel restores the previous mode and theme.

Appearance also offers **Comfortable** and **Compact** rows, with 52 and 40 pixel minimums at default text size, and text presets of **90%, 100%, 110%, and 125%**. Rows grow to fit content while the native window stays fixed and its content scrolls. Existing preferences migrate to Comfortable, 100%, Manual, retaining the selected theme. Theme JSON import/export stays at version 1.

**Create theme** starts with your current palette. You can also duplicate a preset or edit a custom theme. Set its name, dark/light scheme, and nine colors using the swatches or six-digit hex values. Valid changes preview across the application immediately. Save keeps the theme; Cancel or Escape restores your previous selection. Built-in presets remain available. Up to 24 custom themes can be saved.

Themes include background, surface, hover, selection, primary text, secondary text, muted text, border, and accent. Secondary surfaces, icon tiles, focus outlines, keyboard hints, and footer styling follow these colors. Contrast safeguards keep primary text readable on the background, fields, hover, and selection; accent buttons require 4.5:1 contrast, and secondary labels require 3:1 contrast on base surfaces.

Use **Export** to request a JSON download, and **Import** to add a downloaded theme. Files contain colors only: no CSS, scripts, URLs, fonts, or external assets. Import accepts files up to 8 KB. Exported themes do not contain device-specific identifiers.

Example:

```json
{
  "version": 1,
  "name": "Soft Indigo",
  "scheme": "dark",
  "colors": {
    "background": "#181824",
    "surface": "#222233",
    "hover": "#2b2b40",
    "selected": "#36364e",
    "text": "#f5f5fa",
    "secondary": "#acaac2",
    "muted": "#85829d",
    "border": "#37374c",
    "accent": "#b8adf2"
  }
}
```
