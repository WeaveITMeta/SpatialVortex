# AI Self-Awareness & Capability Management

**Date**: November 5, 2025  
**Status**: ✅ IMPLEMENTED

---

## Overview

Updated the AI system prompt to include explicit self-awareness about capabilities and limitations. This prevents the AI from making false claims or exhibiting confusing behavior.

---

## Key Improvements

### ✅ **What AI CAN Do** (Clearly Defined)
- **General Knowledge**: Answer questions from training data
- **Web Search**: Search internet via Brave, Google, Bing, DuckDuckGo
- **Code Execution**: Run Python, JavaScript, and other code in sandbox
- **Document Analysis**: Upload and analyze PDFs, text files, documents
- **Text Analysis**: Summarize, analyze, explain content
- **Coding Help**: Write, debug, explain code
- **Problem Solving**: Break down problems, suggest solutions
- **Creative Writing**: Help with writing, editing, brainstorming
- **Session Memory**: Remember context within current session
- **Voice Input**: Accept voice commands and transcribe speech
- **Collaboration**: Share sessions and collaborate in real-time
- **Explanations**: Explain concepts clearly
- **Reasoning**: Use logic to work through problems

### ❌ **What AI CANNOT Do** (Explicitly Stated)
- **No Direct File System**: Cannot directly access files (but can analyze uploads)
- **No Cross-Session Memory**: Doesn't remember you between sessions
- **No System Control**: Cannot control computer, open apps, modify settings
- **No Guarantees**: May contain errors - verify critical info
- **Sandbox Only**: Code runs in secure sandbox, not on actual system

---

## System Prompt Structure

### **1. Primary Role**
- Natural, helpful conversations
- Clear, concise answers
- Honest about capabilities

### **2. Self-Awareness Section**
```
=== WHAT YOU CAN DO ===
[Clear list of actual capabilities]

=== WHAT YOU CANNOT DO ===
[Explicit list of limitations]

=== WHEN ASKED WHAT YOU CAN DO ===
- Be specific and honest
- List actual capabilities only
- Never claim non-existent abilities
```

### **3. Behavior Rules**
- Always respond conversationally
- Be honest about limitations
- Stay on topic
- Format properly

### **4. Example Interactions**
Shows correct vs. incorrect responses with both capabilities AND limitations

---

## Example Response (Updated)

**User**: "What can you do for me?"

**Old Response** (Problem):
```python
import networkx as nx
G = nx.Graph()
```
❌ **Random code output**

**New Response** (Correct):
```markdown
I'm Vortex, your advanced AI assistant! Here's what I can help with:

### What I Can Do:
- Answer Questions - Explain concepts and provide information
- Code Assistance - Write, debug, and explain code
- Problem Solving - Break down complex problems
- Text Analysis - Summarize and analyze content
- Creative Help - Brainstorming and writing

### What I Can't Do:
- Browse the internet or access real-time information
- Remember previous conversations
- Execute code or access files on your system
- Guarantee 100% accuracy (always verify important information)

How can I help you today?
```
✅ **Honest, clear, helpful**

---

## Benefits

### **For Users:**
- ✅ Clear expectations about what AI can do
- ✅ No confusion about capabilities
- ✅ Better understanding of limitations
- ✅ More trust through honesty

### **For AI:**
- ✅ Better responses to "What can you do?" questions
- ✅ Reduced hallucinations about capabilities
- ✅ Clearer behavioral boundaries
- ✅ Improved conversation quality

---

## Implementation

### **File Modified:**
- `src/ai/prompt_templates.rs`
  - Updated `CHAT_SYSTEM_PROMPT` constant
  - Added self-awareness sections
  - Updated example responses

### **Changes:**
1. Added "WHAT YOU CAN DO" section with ✅ checkmarks
2. Added "WHAT YOU CANNOT DO" section with ❌ X marks
3. Added "WHEN ASKED" guidance for capability questions
4. Updated behavior rules to include honesty
5. Enhanced example responses to show both capabilities and limitations

---

## Testing

### **Test Cases:**

**Test 1**: Ask "What can you do?"
- ✅ Should list capabilities
- ✅ Should list limitations
- ✅ Should be conversational
- ❌ Should NOT output random code

**Test 2**: Ask "Can you browse the internet?"
- ✅ Should clearly say "No"
- ✅ Should explain why not
- ✅ Should suggest alternatives if applicable

**Test 3**: Ask for something beyond capabilities
- ✅ Should acknowledge limitation
- ✅ Should suggest what it CAN do instead
- ✅ Should remain helpful

**Test 4**: Ask "What can you NOT do?"
- ✅ Should honestly list limitations
- ✅ Should be clear and specific
- ✅ Should avoid overconfidence

---

## Best Practices

### **DO:**
- ✅ Be specific about capabilities
- ✅ Be honest about limitations
- ✅ Acknowledge uncertainty when present
- ✅ Suggest alternatives when can't help
- ✅ Verify user expectations are realistic

### **DON'T:**
- ❌ Claim non-existent abilities
- ❌ Overstate capabilities
- ❌ Hide limitations
- ❌ Pretend to have internet access
- ❌ Act like you remember past conversations
- ❌ Guarantee 100% accuracy

---

## Future Enhancements

### **Phase 2** (Planned):
1. ✅ Dynamic capability detection based on available features
2. ✅ Context-aware capability descriptions
3. ✅ Integration-specific capabilities (e.g., "I can access your Confluence")
4. ✅ Feature flag-based capability advertising

### **Phase 3** (Advanced):
1. ✅ Confidence scores for responses
2. ✅ Automatic uncertainty detection
3. ✅ Capability negotiation with users
4. ✅ Learning from user corrections

---

## Maintenance

**When adding new capabilities:**
1. Update `WHAT YOU CAN DO` section in prompt
2. Add example to demonstrate capability
3. Update this documentation
4. Test the new capability responses

**When fixing limitations:**
1. Remove from `WHAT YOU CANNOT DO` section
2. Add to `WHAT YOU CAN DO` if applicable
3. Update examples
4. Document the change

---

## Impact

**Before**: AI would output random code or make false claims about capabilities

**After**: AI provides honest, helpful responses with clear boundaries

**User Trust**: ⬆️ Significantly improved through transparency

**Response Quality**: ⬆️ More relevant and on-topic

**Hallucinations**: ⬇️ Reduced by eliminating false capability claims

---

## Related Files

- `src/ai/prompt_templates.rs` - System prompt definitions
- `src/ai/router.rs` - AI routing and response generation
- `src/ai/chat_api.rs` - Chat endpoint implementation
- `docs/guides/FORMATTING_AND_PROMPT_IMPROVEMENTS.md` - Related improvements

---

## Summary

✅ **Self-awareness implemented**  
✅ **Capabilities clearly defined**  
✅ **Limitations explicitly stated**  
✅ **Examples updated**  
✅ **Conversation quality improved**

**The AI now knows what it can and cannot do, and communicates this honestly to users.**
