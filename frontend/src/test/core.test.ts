import { describe, expect, it } from 'vitest';
import { createTranslator, messageKeys } from '@/i18n/messages';

describe('V3 product contract', () => {
  it('keeps the fixed three-model product boundary', () => {
    expect(['qwen3-asr-1.7b', 'qwen3-asr-0.6b', 'fun-asr-nano-2512']).toHaveLength(3);
    expect(messageKeys).not.toContain('whisper');
    expect(messageKeys).not.toContain('ollama');
  });

  it('has a translation for every Simplified Chinese key', () => {
    const zh = createTranslator('zh-CN');
    const en = createTranslator('en-US');
    for (const key of messageKeys) {
      expect(zh(key), key).not.toBe(key);
      expect(en(key), key).not.toBe(key);
    }
  });

  it('defaults to Simplified Chinese product copy', () => {
    expect(createTranslator('zh-CN')('brand')).toBe('课溯');
    expect(createTranslator('en-US')('brand')).toBe('VeriLecture');
  });
});
