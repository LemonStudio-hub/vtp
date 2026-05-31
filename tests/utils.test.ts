import { describe, it, expect, vi } from 'vitest';
import {
  formatBytes,
  formatNumber,
  formatSpeed,
  formatTime,
  generateNodeId,
  sleep,
  debounce
} from '../src/utils/index';

describe('formatBytes', () => {
  it('formats zero bytes', () => {
    expect(formatBytes(0)).toBe('0 B');
  });

  it('formats bytes', () => {
    expect(formatBytes(512)).toBe('512 B');
  });

  it('formats kilobytes', () => {
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(1536)).toBe('1.5 KB');
  });

  it('formats megabytes', () => {
    expect(formatBytes(1048576)).toBe('1 MB');
    expect(formatBytes(5242880)).toBe('5 MB');
  });

  it('formats gigabytes', () => {
    expect(formatBytes(1073741824)).toBe('1 GB');
  });

  it('formats fractional values correctly', () => {
    expect(formatBytes(1025)).toBe('1 KB');
    expect(formatBytes(2048)).toBe('2 KB');
  });
});

describe('formatNumber', () => {
  it('formats zero', () => {
    expect(formatNumber(0)).toBe('0');
  });

  it('formats small numbers', () => {
    expect(formatNumber(42)).toBe('42');
  });

  it('formats thousands', () => {
    const result = formatNumber(1234567);
    expect(result).toContain('1');
    expect(result).toContain('234');
    expect(result).toContain('567');
  });

  it('formats decimals', () => {
    const result = formatNumber(1234.567);
    expect(result).toContain('1');
    expect(result).toContain('234');
  });
});

describe('formatSpeed', () => {
  it('formats values below 1000', () => {
    expect(formatSpeed(500)).toBe('500');
    expect(formatSpeed(0)).toBe('0');
    expect(formatSpeed(999)).toBe('999');
  });

  it('formats thousands', () => {
    expect(formatSpeed(1000)).toBe('1.0K');
    expect(formatSpeed(1500)).toBe('1.5K');
    expect(formatSpeed(999999)).toBe('1000.0K');
  });

  it('formats millions', () => {
    expect(formatSpeed(1000000)).toBe('1.0M');
    expect(formatSpeed(1500000)).toBe('1.5M');
    expect(formatSpeed(2500000)).toBe('2.5M');
  });

  it('formats boundary values', () => {
    expect(formatSpeed(999)).toBe('999');
    expect(formatSpeed(1000)).toBe('1.0K');
    expect(formatSpeed(999999)).toBe('1000.0K');
    expect(formatSpeed(1000000)).toBe('1.0M');
  });
});

describe('formatTime', () => {
  it('formats zero seconds', () => {
    expect(formatTime(0)).toBe('00:00:00');
  });

  it('formats seconds only', () => {
    expect(formatTime(30)).toBe('00:00:30');
    expect(formatTime(59)).toBe('00:00:59');
  });

  it('formats minutes and seconds', () => {
    expect(formatTime(60)).toBe('00:01:00');
    expect(formatTime(61)).toBe('00:01:01');
    expect(formatTime(90)).toBe('00:01:30');
  });

  it('formats hours', () => {
    expect(formatTime(3600)).toBe('01:00:00');
    expect(formatTime(3661)).toBe('01:01:01');
  });

  it('formats large values', () => {
    expect(formatTime(86400)).toBe('24:00:00');
    expect(formatTime(100000)).toBe('27:46:40');
  });

  it('pads single digits', () => {
    expect(formatTime(1)).toBe('00:00:01');
    expect(formatTime(10)).toBe('00:00:10');
  });
});

describe('generateNodeId', () => {
  it('returns 8 character string', () => {
    expect(generateNodeId()).toHaveLength(8);
  });

  it('contains only hex characters', () => {
    const id = generateNodeId();
    expect(id).toMatch(/^[0-9a-f]{8}$/);
  });

  it('generates different IDs', () => {
    const id1 = generateNodeId();
    const id2 = generateNodeId();
    expect(id1).not.toBe(id2);
  });
});

describe('sleep', () => {
  it('resolves after specified time', async () => {
    const start = Date.now();
    await sleep(50);
    const elapsed = Date.now() - start;
    expect(elapsed).toBeGreaterThanOrEqual(40);
  });

  it('resolves with void', async () => {
    const result = await sleep(1);
    expect(result).toBeUndefined();
  });
});

describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('delays function execution', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledOnce();
  });

  it('cancels previous call on rapid invocations', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    debounced();
    debounced();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledOnce();
  });

  it('passes arguments to the debounced function', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('arg1', 'arg2');
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledWith('arg1', 'arg2');
  });

  it('uses last arguments on rapid calls', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('first');
    debounced('second');
    debounced('third');

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledWith('third');
  });

  it('allows multiple executions after wait period', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('first');
    vi.advanceTimersByTime(100);

    debounced('second');
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledTimes(2);
  });
});
