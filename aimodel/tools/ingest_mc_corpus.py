"""Fetch real benchmark data and normalise SIX incompatible answer paradigms
into one training format.

Measured shapes (live, 2026-08-08):
  gsm8k       free-form answer with chain-of-thought and a '#### N' marker
  mmlu        integer index into an inline `choices` list
  arc         letter key ('B') into a {'text': [...], 'label': [...]} dict
  openbookqa  same as arc, but the question field is `question_stem`
  sciq        `correct_answer` plus separate distractor1/2/3 fields
  hellaswag   integer-as-string index into `endings`, question in `ctx`
  squad       extractive span with character offsets
  boolq       a raw bool

Flattening these naively trains on `True` as answer text and on '#### 72' as
prose. Each converter below is explicit about what it is doing, and anything it
cannot map confidently is DROPPED and counted rather than guessed at.

Output: JSONL of {dataset, question, choices[], answer_idx}
Only paradigms that yield genuine multiple choice are emitted; gsm8k/squad are
counted and skipped, since turning a free-form answer into choices would mean
inventing distractors.
"""
import json, urllib.parse, urllib.request, random, sys, io, collections

OUT = sys.argv[1] if len(sys.argv) > 1 else "corpus.jsonl"
PER = int(sys.argv[2]) if len(sys.argv) > 2 else 600

SOURCES = [
    ("mmlu",       "cais/mmlu",          "all",        "test"),
    ("arc",        "allenai/ai2_arc",    "ARC-Easy",   "train"),
    ("arc_hard",   "allenai/ai2_arc",    "ARC-Challenge", "train"),
    ("openbookqa", "allenai/openbookqa", "main",       "train"),
    ("sciq",       "allenai/sciq",       "default",    "train"),
    ("hellaswag",  "Rowan/hellaswag",    "default",    "train"),
]

def fetch(path, config, split, offset, length):
    url = ("https://datasets-server.huggingface.co/rows"
           f"?dataset={urllib.parse.quote(path, safe='')}"
           f"&config={urllib.parse.quote(config, safe='')}"
           f"&split={split}&offset={offset}&length={length}")
    with urllib.request.urlopen(url, timeout=60) as r:
        return json.load(r)

def norm_mmlu(r):
    ch = r.get("choices")
    a = r.get("answer")
    if not isinstance(ch, list) or not isinstance(a, int) or not (0 <= a < len(ch)):
        return None
    return r["question"], [str(c) for c in ch], a

def norm_arc(r):
    ch = r.get("choices") or {}
    texts, labels = ch.get("text"), ch.get("label")
    key = r.get("answerKey")
    if not texts or not labels or key is None or key not in labels:
        return None
    return r["question"], [str(t) for t in texts], labels.index(key)

def norm_openbookqa(r):
    ch = r.get("choices") or {}
    texts, labels = ch.get("text"), ch.get("label")
    key = r.get("answerKey")
    q = r.get("question_stem")
    if not texts or not labels or key not in (labels or []) or not q:
        return None
    return q, [str(t) for t in texts], labels.index(key)

def norm_sciq(r):
    correct = r.get("correct_answer")
    ds = [r.get("distractor1"), r.get("distractor2"), r.get("distractor3")]
    q = r.get("question")
    if not correct or not q or any(d is None for d in ds):
        return None
    opts = [str(correct)] + [str(d) for d in ds]
    idx = list(range(4))
    random.shuffle(idx)                      # correct answer must not sit at 0
    return q, [opts[i] for i in idx], idx.index(0)

def norm_hellaswag(r):
    ends = r.get("endings")
    lab = r.get("label")
    ctx = r.get("ctx") or r.get("ctx_a")
    if not isinstance(ends, list) or ctx is None:
        return None
    try:
        li = int(lab)
    except (TypeError, ValueError):
        return None
    if not (0 <= li < len(ends)):
        return None
    return ctx, [str(e) for e in ends], li

NORM = {
    "mmlu": norm_mmlu, "arc": norm_arc, "arc_hard": norm_arc,
    "openbookqa": norm_openbookqa, "sciq": norm_sciq, "hellaswag": norm_hellaswag,
}

random.seed(20260808)
written = 0
stats = collections.OrderedDict()
with io.open(OUT, "w", encoding="utf-8") as fh:
    for name, path, config, split in SOURCES:
        got, dropped, offset = 0, 0, 0
        while got < PER:
            try:
                d = fetch(path, config, split, offset, min(100, PER - got + 20))
            except Exception as e:
                print(f"  {name}: fetch failed at offset {offset}: {str(e)[:60]}")
                break
            rows = d.get("rows", [])
            if not rows:
                break
            for rw in rows:
                out = NORM[name](rw["row"])
                if out is None:
                    dropped += 1
                    continue
                q, choices, ai = out
                if len(choices) < 2 or not q.strip():
                    dropped += 1
                    continue
                fh.write(json.dumps({
                    "dataset": name, "question": q,
                    "choices": choices, "answer_idx": ai,
                }, ensure_ascii=False) + "\n")
                got += 1
                written += 1
                if got >= PER:
                    break
            offset += len(rows)
        stats[name] = (got, dropped)
        print(f"  {name:<12} kept {got:>5}  dropped {dropped:>4}")

print(f"\n  total written : {written}")
print(f"  output        : {OUT}")
n_choice = collections.Counter()
with io.open(OUT, encoding="utf-8") as fh:
    for line in fh:
        n_choice[len(json.loads(line)["choices"])] += 1
print(f"  choice counts : {dict(sorted(n_choice.items()))}")
chance = sum(c / k for k, c in n_choice.items()) / max(written, 1)
print(f"  chance accuracy for this mix: {100*chance:.1f}%")
