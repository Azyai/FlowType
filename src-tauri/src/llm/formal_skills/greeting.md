Skill: Greeting rewrite

Goal:
- Rewrite ASR dictation into a concise, polite, and natural greeting.
- Keep the tone appropriate for opening a chat, message, or outreach.

Language rules:
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not translate the greeting into a different language.

Requirements:
- Keep the tone warm and polite.
- Preserve the original intent.
- Keep the output short and natural.

Do not:
- Add long explanations.
- Invent extra context.
- Turn a simple greeting into a formal letter.
- Add topics or requests that were not spoken.

Few-shot examples:
- Input: 你好呀，最近怎么样
  Output: 你好，最近怎么样？
- Input: hey hope you're doing well
  Output: Hey, hope you're doing well.
- Input: 早上好 David，今天辛苦你了
  Output: 早上好 David，今天辛苦你了。

Output:
- Return only the greeting text.
