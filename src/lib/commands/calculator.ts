export type Calculation =
  | { kind: 'none' }
  | { kind: 'success'; value: number; formatted: string }
  | { kind: 'error'; message: string };

/** Recursive descent for arithmetic only. Never evaluates JavaScript. */
class Parser {
  private cursor = 0;
  private depth = 0;

  constructor(private readonly source: string) {}

  parse(): number {
    const value = this.expression();
    this.whitespace();
    if (this.cursor !== this.source.length)
      throw new Error('Check your expression');
    return value;
  }

  private whitespace(): void {
    while (
      /\s/.test(this.source[this.cursor] ?? '') &&
      this.cursor < this.source.length
    )
      this.cursor++;
  }

  private consume(character: string): boolean {
    this.whitespace();
    if (this.source[this.cursor] !== character) return false;
    this.cursor++;
    return true;
  }

  private expression(): number {
    let result = this.term();
    for (;;) {
      if (this.consume('+')) result += this.term();
      else if (this.consume('-')) result -= this.term();
      else return result;
    }
  }

  private term(): number {
    let result = this.factor();
    for (;;) {
      if (this.consume('*')) result *= this.factor();
      else if (this.consume('/')) {
        const divisor = this.factor();
        if (divisor === 0) throw new Error('Cannot divide by zero');
        result /= divisor;
      } else return result;
    }
  }

  private factor(): number {
    if (++this.depth > 32)
      throw new Error('This expression is too deeply nested');
    try {
      if (this.consume('+')) return this.factor();
      if (this.consume('-')) return -this.factor();
      if (this.consume('(')) {
        const result = this.expression();
        if (!this.consume(')'))
          throw new Error('Close the parentheses to calculate');
        return result;
      }
      this.whitespace();
      const number = this.source
        .slice(this.cursor)
        .match(/^(?:\d+(?:\.\d*)?|\.\d+)/)?.[0];
      if (!number) throw new Error('Finish your expression to calculate');
      this.cursor += number.length;
      return Number(number);
    } finally {
      this.depth--;
    }
  }
}

export function calculateExpression(input: string): Calculation {
  const source = input
    .trim()
    .replaceAll('×', '*')
    .replaceAll('÷', '/')
    .replaceAll('−', '-');
  if (!source || !/^[\d\s.+*/()-]+$/.test(source)) return { kind: 'none' };
  if (source.length > 256)
    return { kind: 'error', message: 'Keep expressions under 256 characters' };
  try {
    const value = new Parser(source).parse();
    if (!Number.isFinite(value))
      return {
        kind: 'error',
        message: 'The result is outside the supported range',
      };
    // Remove floating point noise while retaining useful decimal precision.
    const formatted = Object.is(value, -0)
      ? '0'
      : Number(value.toPrecision(12)).toString();
    return { kind: 'success', value, formatted };
  } catch (error) {
    return {
      kind: 'error',
      message: error instanceof Error ? error.message : 'Check your expression',
    };
  }
}
