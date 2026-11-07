export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
}

export interface ConversationTurn {
  prompt: string;
  text: string;
  status: string;
  error: string;
}

/** Keep whole, completed exchanges in order; never replay errors or thinking. */
export function conversationHistory(
  turns: readonly ConversationTurn[],
): ConversationMessage[] {
  const pairs: ConversationMessage[][] = [];
  let bytes = 0;
  const encoder = new TextEncoder();
  for (const turn of [...turns].reverse()) {
    if (turn.error || turn.status !== 'Complete' || !turn.text) continue;
    const pair: ConversationMessage[] = [
      { role: 'user', content: turn.prompt },
      { role: 'assistant', content: turn.text },
    ];
    const size = pair.reduce(
      (total, message) => total + encoder.encode(message.content).length,
      0,
    );
    if (pairs.length === 16 || bytes + size > 256_000) break;
    pairs.push(pair);
    bytes += size;
  }
  return pairs.reverse().flat();
}
