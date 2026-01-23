# Images and Citations Test

## 🖼️ Image Support

Test images with this markdown:

```markdown
![Alt text](https://via.placeholder.com/600x400/60a5fa/ffffff?text=Sample+Image)

Or inline: Here's an image ![icon](https://via.placeholder.com/50x50) in text.
```

**Features:**
- Responsive (max-width: 100%)
- Rounded corners
- Shadow effects
- Border styling
- Centered in paragraphs

---

## 📚 Citation Support

### Method 1: Using `<cite>` tag

```markdown
<cite>Smith, J. (2024). "Advanced AI Systems." Journal of Machine Learning, 15(3), 245-260.</cite>
```

**Renders as:**
- 📚 icon prefix
- Blue left border
- Light blue background
- Italic text

### Method 2: Superscript references

```markdown
This is a statement that needs citation.<sup>[1]</sup>

Another fact requiring a source.<sup>[2]</sup>

---

**References:**

<cite>[1] Jones, A. (2023). "Deep Learning Fundamentals."</cite>
<cite>[2] Williams, B. (2024). "Neural Network Architecture."</cite>
```

### Method 3: Inline citations

```markdown
According to recent research (Johnson et al., 2024)<sup>📚</sup>, AI systems...
```

---

## 🎨 Styling

### Images:
- ✅ Auto-resize to container
- ✅ Rounded 8px borders  
- ✅ Shadow: `0 4px 12px rgba(0, 0, 0, 0.3)`
- ✅ Border: `1px solid rgba(255, 255, 255, 0.1)`

### Citations:
- ✅ Blue theme (`#60a5fa`)
- ✅ 📚 Book emoji prefix
- ✅ Blue left border (3px)
- ✅ Light background
- ✅ Italic styling

### Superscripts:
- ✅ Blue color
- ✅ Bold weight
- ✅ Cursor: help (shows question mark on hover)

---

## 🧪 Test Examples

### Example 1: Academic Paper

```markdown
## The Impact of AI on Modern Computing

Recent studies have shown significant improvements in computational efficiency<sup>[1]</sup>. 
The following diagram illustrates the architecture:

![AI Architecture](https://via.placeholder.com/800x400/60a5fa/ffffff?text=AI+Architecture+Diagram)

<cite>[1] Thompson, R. (2024). "Computational Efficiency in Modern AI." Nature Machine Intelligence, 6(2), 123-134.</cite>
```

### Example 2: Technical Documentation

```markdown
## System Overview

The vortex algorithm operates as shown below:

![Vortex Flow](https://via.placeholder.com/600x300/3b82f6/ffffff?text=Vortex+Flow+Diagram)

Implementation details can be found in the technical specification<sup>①</sup>.

<cite>① Internal Documentation: Vortex Algorithm v2.0 (2024)</cite>
```

### Example 3: Multiple Citations

```markdown
## Literature Review

Machine learning has evolved significantly<sup>[1]</sup>, with deep learning 
emerging as a dominant paradigm<sup>[2]</sup>. Recent advances in transformer 
architectures<sup>[3]</sup> have revolutionized the field.

**References:**

<cite>[1] LeCun, Y., Bengio, Y., & Hinton, G. (2015). "Deep learning." Nature, 521(7553), 436-444.</cite>

<cite>[2] Goodfellow, I., Bengio, Y., & Courville, A. (2016). "Deep Learning." MIT Press.</cite>

<cite>[3] Vaswani, A., et al. (2017). "Attention is all you need." NeurIPS, 30.</cite>
```

---

## ✨ Usage Tips

### For Images:
1. Use descriptive alt text
2. Images auto-center in paragraphs
3. Responsive by default
4. Supported formats: JPG, PNG, GIF, SVG, WebP

### For Citations:
1. Use `<cite>` for formal citations
2. Use `<sup>` for reference numbers
3. Group citations at end of section
4. Blue theme matches overall design

---

## 🎯 Testing Checklist

- [ ] Images load and display properly
- [ ] Images are responsive
- [ ] Image borders and shadows appear
- [ ] Citations show 📚 icon
- [ ] Citations have blue left border
- [ ] Superscripts are blue and bold
- [ ] Hover over sup shows help cursor
- [ ] Multiple citations format correctly
- [ ] Inline images work
- [ ] Block images center properly
