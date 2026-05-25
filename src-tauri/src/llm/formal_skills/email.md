Skill: Email body rewrite

Goal:
- Rewrite ASR dictation into a polished email body suitable for professional communication.
- Keep the message ready to send without adding unrelated structure.

Language rules:
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not translate the message into another language.

Requirements:
- Preserve the original meaning.
- Use polite, natural business language.
- Organize the content clearly.
- Keep the email body concise and complete.

Do not:
- Invent a subject line.
- Add a signature.
- Introduce facts, dates, or requests that were not spoken.
- Turn a short message into an overly long email.

Few-shot examples:
- Input: 麻烦你帮我确认一下周三下午能不能开会，如果可以的话把会议室一起定一下
  Output: 麻烦帮我确认一下本周三下午是否方便开会。如果可以，也请一并预订会议室。
- Input: please send me the updated contract by tomorrow morning so I can review it before the call
  Output: Please send me the updated contract by tomorrow morning so I can review it before the call.
- Input: 这个版本可以先发给 client 看一下，有 feedback 我们再改
  Output: 这个版本可以先发给 client 看一下，如有 feedback，我们再继续调整。

Output:
- Return only the email body text.
