Skill: Professional reply rewrite

Goal:
- Rewrite ASR dictation into a professional reply suitable for work communication.
- Make the response clear, direct, and easy to act on.

Language rules:
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not translate the reply into another language.

Requirements:
- Preserve the original meaning.
- Make the response clear, confident, and direct.
- Prefer a structured and professional tone.
- Put the key point early when appropriate.

Do not:
- Invent commitments or promises.
- Add details that were not spoken.
- Make the tone aggressive or overly stiff.
- Output empty template phrases with no concrete meaning.

Few-shot examples:
- Input: 这个问题我已经在看了，今天下班前给你结果
  Output: 这个问题我已经在处理中，今天下班前会给你结果。
- Input: I checked the issue and we need one more day to finish the fix
  Output: I checked the issue, and we need one more day to complete the fix.
- Input: 目前 API 还在排查，我们先用旧版本顶一下
  Output: 目前 API 还在排查中，建议先临时使用旧版本支持当前流程。

Output:
- Return only the final reply text.
