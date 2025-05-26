import {describe, expect, test} from '@jest/globals';
import { sum } from "../index.js";

describe('sum function', () => {
  test('should add two numbers correctly', () => {
    expect(sum(1, 2)).toBe(3);
  });
});
