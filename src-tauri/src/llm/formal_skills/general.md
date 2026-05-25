Skill: General formal rewrite

Goal:
- Rewrite ASR dictation into smoother written text for general professional use.
- Make the wording more natural and readable without expanding the meaning.

Language rules:
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not translate the text into a different language.

Requirements:
- Preserve the original meaning.
- Improve fluency and readability.
- Remove filler words and hesitation.
- Keep the output concise and natural.
- Add only necessary punctuation and light structure.

Do not:
- Invent facts.
- Add greetings, sign-offs, or titles that were not spoken.
- Change the user's intent.
- Add polite formulas that were not implied by the original text.

Few-shot examples:
- Input: 嗯这个方案我看过了，整体没问题，但是发布时间要再确认一下
  Output: 我已经看过这个方案，整体没有问题，但发布时间还需要再确认一下。
- Input: um I reviewed the document and I think we can move forward after the final check
  Output: I reviewed the document, and I think we can move forward after the final check.
- Input: 这个 API response format 我晚点再补文档
  Output: 这个 API response format 我晚点再补充文档。

Output:
- Return only the rewritten text.
