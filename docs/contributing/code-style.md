# SpatialVortex Documentation Style Guide
## Writing Standards for Clear, Professional Documentation

**Version**: 1.0  
**Date**: October 23, 2025  
**Applies To**: All documentation, code comments, commit messages

---

## 🎯 Core Principles

### **1. Avoid Abbreviations**

Always spell out terms on first use. Abbreviations may be used in parentheses for subsequent references.

**Good Examples**:
```markdown
The Retrieval-Augmented Generation (RAG) system queries external databases.
After setup, the RAG pipeline processes requests efficiently.
```

**Bad Examples**:
```markdown
The RAG system queries databases.  ❌ (No explanation of RAG)
The API calls the ML model.        ❌ (Multiple unexplained abbreviations)
```

---

### **2. Proper Case and Capitalization**

Use correct capitalization for technical terms, proper nouns, and titles.

**Good Examples**:
```markdown
- Application Programming Interface (API)
- Graphics Processing Unit (GPU)
- Central Processing Unit (CPU)
- Virtual Machine (VM)
- Machine Learning (ML)
- Natural Language Processing (NLP)
```

**Bad Examples**:
```markdown
- application programming interface  ❌ (Incorrect case)
- GPU                               ❌ (No expansion on first use)
- vm instance                       ❌ (Improper case)
```

---

### **3. Clear Semantics**

Use precise, unambiguous language. Avoid jargon without explanation.

**Good Examples**:
```markdown
"The hypervisor allocates CPU cores to each Virtual Machine instance."
"The inference engine processes semantic queries through the flux matrix."
```

**Bad Examples**:
```markdown
"The hypervisor allocs cores to VMs."           ❌ (Informal, abbreviated)
"The engine does stuff with the matrix."       ❌ (Vague, imprecise)
```

---

## 📋 Common Abbreviations to Expand

### **Technical Terms**
| Abbreviation | Full Term | Usage |
|--------------|-----------|-------|
| **API** | Application Programming Interface | Always expand on first use |
| **CPU** | Central Processing Unit | Always expand on first use |
| **GPU** | Graphics Processing Unit | Always expand on first use |
| **RAM** | Random Access Memory | Always expand on first use |
| **SSD** | Solid State Drive | Always expand on first use |
| **HDD** | Hard Disk Drive | Always expand on first use |
| **UI** | User Interface | Always expand on first use |
| **UX** | User Experience | Always expand on first use |
| **CLI** | Command Line Interface | Always expand on first use |
| **GUI** | Graphical User Interface | Always expand on first use |

### **AI/ML Terms**
| Abbreviation | Full Term | Usage |
|--------------|-----------|-------|
| **AI** | Artificial Intelligence | Common, but expand on first use |
| **ML** | Machine Learning | Always expand on first use |
| **NLP** | Natural Language Processing | Always expand on first use |
| **LLM** | Large Language Model | Always expand on first use |
| **RAG** | Retrieval-Augmented Generation | Always expand on first use |
| **LoRA** | Low-Rank Adaptation | Always expand on first use |
| **PEFT** | Parameter-Efficient Fine-Tuning | Always expand on first use |
| **RLHF** | Reinforcement Learning from Human Feedback | Always expand on first use |

### **SpatialVortex-Specific Terms**
| Abbreviation | Full Term | Usage |
|--------------|-----------|-------|
| **ASI** | Artificial Superintelligence | Project name - acceptable after first expansion |
| **VMAI** | Virtual Machine Artificial Intelligence | Expand on first use, then acceptable |
| **ELP** | Ethos, Logos, Pathos | Project-specific - expand on first use |
| **CRUD** | Create, Read, Update, Delete | Common pattern - expand on first use |

### **Acceptable Without Expansion**
These are universally understood and don't require expansion:
- **JSON** (JavaScript Object Notation)
- **HTTP** (Hypertext Transfer Protocol)
- **URL** (Uniform Resource Locator)
- **SQL** (Structured Query Language)
- **PDF** (Portable Document Format)
- **CSV** (Comma-Separated Values)

---

## ✍️ Writing Style

### **Tone**
- **Professional** but approachable
- **Technical** without being pedantic
- **Clear** without being condescending

### **Voice**
- Use **active voice** when possible
  - Good: "The system processes requests"
  - Bad: "Requests are processed by the system"

- Use **second person** for user-facing docs
  - Good: "You can configure the hypervisor"
  - Bad: "One can configure the hypervisor"

### **Tense**
- **Present tense** for current functionality
  - Good: "The system runs at 1000 Hz"
  - Bad: "The system will run at 1000 Hz" (unless future feature)

### **Structure**
- **Short paragraphs** (3-5 sentences max)
- **Bulleted lists** for multiple items
- **Code examples** to illustrate concepts
- **Headings** for clear section breaks

---

## 📝 Document Structure

### **Every Document Should Have**:
1. **Title** (H1) - Clear, descriptive
2. **Subtitle** (H2) - Explains purpose
3. **Metadata** - Version, date, status
4. **Introduction** - What, why, who
5. **Body** - Main content with clear sections
6. **Examples** - Code samples where applicable
7. **Next Steps** - What to read/do next

### **Template**:
```markdown
# Document Title
## Brief Description

**Version**: X.X  
**Date**: Month Day, Year  
**Status**: Draft | Review | Complete

---

## 🎯 Overview

Brief introduction to the topic...

## 📚 Main Content

### Section 1
Content with examples...

### Section 2
More content...

## 📋 Examples

```rust
// Code example
```

## 🔗 Related Documentation
- [Link to related doc](path/to/doc.md)

---

**Next Steps**: What the reader should do next
```

---

## 🔤 Capitalization Rules

### **Titles and Headings**
Use **Title Case** for main headings:
- Good: "Virtual Machine Artificial Intelligence"
- Bad: "virtual machine artificial intelligence"

### **Technical Terms**
Follow standard technical capitalization:
- Rust (language name)
- PostgreSQL (product name)
- tokio (library name - lowercase per convention)
- GitHub (company name)

### **Project Terms**
- SpatialVortex (product name - PascalCase)
- Artificial Superintelligence (concept - Title Case)
- flux matrix (system component - lowercase when not starting sentence)

---

## 🔢 Numbers and Units

### **Numbers**
- Spell out zero through nine in prose
  - Good: "The system has three phases"
  - Exception: Technical specs use digits: "3 CPU cores"

- Use digits for 10 and above
  - Good: "The benchmark includes 200 tasks"

### **Units**
- Use standard abbreviations WITH explanation:
  - Memory: "8 gigabytes (GB) of RAM"
  - Time: "10 milliseconds (ms)"
  - Frequency: "1000 Hertz (Hz)"

- After first use, abbreviation alone is acceptable:
  - "The system allocates 4GB initially, then scales to 16GB under load."

---

## 💻 Code and Technical Elements

### **Code Blocks**
Always specify language for syntax highlighting:
```rust
// Good - language specified
pub fn example() {}
```

### **Inline Code**
Use backticks for:
- Function names: `create_instance()`
- Variable names: `vm_id`
- File paths: `src/main.rs`
- Commands: `cargo build`
- Short code snippets: `let x = 5;`

### **File Paths**
Use forward slashes for consistency:
- Good: `docs/architecture/ASI_ARCHITECTURE.md`
- Acceptable on Windows: `e:\Libraries\SpatialVortex\`

---

## 🔗 Links and References

### **Internal Links**
Use relative paths:
```markdown
See [Sacred Positions](architecture/SACRED_POSITIONS.md)
```

### **External Links**
Use descriptive text, not URLs:
```markdown
Good: Read the [Rust documentation](https://doc.rust-lang.org)
Bad: See https://doc.rust-lang.org
```

### **Cross-References**
Link to related documentation:
```markdown
**Related**:
- [ASI Architecture](ASI_ARCHITECTURE.md)
- [Sacred Positions](SACRED_POSITIONS.md)
```

---

## 📐 Formatting Standards

### **Emphasis**
- **Bold** for important terms on first introduction
- *Italic* for emphasis or foreign terms
- `Code formatting` for technical elements

### **Lists**
- Use **bullets** for unordered items
- Use **numbers** for sequential steps
- Use **tables** for comparisons

### **Emoji Usage**
Use sparingly for visual organization:
- ✅ Acceptable for status indicators
- 🎯 Acceptable for section markers
- ❌ Don't overuse or use for decoration

---

## ❌ Common Mistakes to Avoid

### **1. Unexplained Abbreviations**
```markdown
❌ "The ML model uses NLP for RAG."
✅ "The Machine Learning (ML) model uses Natural Language Processing (NLP) 
   for Retrieval-Augmented Generation (RAG)."
```

### **2. Inconsistent Capitalization**
```markdown
❌ "the API calls the Ml Model"
✅ "The API calls the Machine Learning model"
```

### **3. Vague Language**
```markdown
❌ "The system does stuff with data"
✅ "The system processes semantic queries through the flux matrix"
```

### **4. Overly Technical Without Context**
```markdown
❌ "Use tokio::spawn for async ops"
✅ "Use `tokio::spawn()` to execute asynchronous operations concurrently"
```

### **5. Wall of Text**
```markdown
❌ One huge paragraph with no breaks...

✅ Short paragraphs

   With clear breaks

   And proper structure
```

---

## 📚 Examples

### **Good Documentation Example**:

```markdown
# Virtual Machine Artificial Intelligence

The Virtual Machine Artificial Intelligence (VMAI) system provides isolated
execution environments for Artificial Intelligence (AI) workloads. Each 
Virtual Machine (VM) instance receives dedicated Central Processing Unit (CPU)
cores, Random Access Memory (RAM), and optionally Graphics Processing Unit (GPU)
resources.

## Resource Allocation

The hypervisor allocates resources based on workload type:

- **Creative tasks**: 4 CPU cores, 8 gigabytes (GB) RAM
- **Analytical tasks**: 8 CPU cores, 16GB RAM  
- **Synthesis tasks**: 4 CPU cores, 8GB RAM

After initial allocation, the VMAI system can scale resources dynamically
based on demand.
```

### **Bad Documentation Example**:

```markdown
# VMAI

VMAI provides exec envs for AI workloads. Each VM gets CPU, RAM, and maybe GPU.

## Resources

Hypervisor allocs based on type:
- Creative: 4c, 8GB
- Analytical: 8c, 16GB
- Synthesis: 4c, 8GB

Can scale dynamically.
```

---

## ✅ Checklist for New Documentation

Before submitting documentation, verify:

- [ ] All abbreviations explained on first use
- [ ] Proper capitalization throughout
- [ ] Clear, unambiguous language
- [ ] Code blocks have language specified
- [ ] Links use descriptive text
- [ ] Headings use Title Case
- [ ] Lists are formatted correctly
- [ ] Examples are included where helpful
- [ ] Technical terms are explained
- [ ] Structure follows template
- [ ] Related docs are cross-referenced
- [ ] No walls of text (paragraphs broken up)
- [ ] Active voice used where possible
- [ ] Professional but approachable tone

---

## 🔄 Review Process

### **Self-Review**
1. Read document aloud
2. Check against this style guide
3. Run spell check
4. Verify all links work
5. Test all code examples

### **Peer Review**
Request review focusing on:
- Technical accuracy
- Clarity for target audience
- Adherence to style guide
- Completeness

---

## 🎓 Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Microsoft Writing Style Guide](https://learn.microsoft.com/en-us/style-guide/welcome/)
- [Google Developer Documentation Style Guide](https://developers.google.com/style)
- [Write the Docs](https://www.writethedocs.org/)

---

**Status**: Official Style Guide  
**Enforcement**: All new documentation  
**Updates**: Review quarterly

**Remember**: Clear documentation is as important as clear code. Take the time to write it well.
