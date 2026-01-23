# Favicon Instructions

## Current Setup

The site icon is now a **blue spiral** in SVG format.

### Files:
- `favicon.svg` - Blue gradient spiral (primary, modern browsers)
- `favicon.ico` - Fallback for older browsers

### Blue Spiral SVG
The favicon.svg uses the Vortex blue gradient:
- Light blue: #60a5fa
- Dark blue: #3b82f6

### To Update favicon.ico

Since .ico files are binary, use one of these methods:

**Option 1: Online Converter**
1. Go to https://favicon.io/favicon-converter/
2. Upload the `favicon.svg` file
3. Download the generated `favicon.ico`
4. Replace `web/static/favicon.ico`

**Option 2: Use ImageMagick**
```bash
convert favicon.svg -define icon:auto-resize=16,32,48 favicon.ico
```

**Option 3: Use GIMP**
1. Open `favicon.svg` in GIMP
2. Export as `.ico`
3. Choose sizes: 16x16, 32x32, 48x48

### Quick Emoji Alternative

For a quick fix, the 🌀 emoji works great as a temporary favicon!
