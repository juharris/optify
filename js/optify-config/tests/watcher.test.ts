import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { OptionsWatcher } from '../index';

const MODIFICATION_DEBOUNCE_MS = 1000;
const RETRY_DELAY = MODIFICATION_DEBOUNCE_MS * 2 + 100;
const MAX_RETRY_ATTEMPTS = 10;

describe("OptionsWatcher", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-watcher-test-'));
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test("listener is called when a file is modified", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);
    const builtTime = watcher.lastModified();
    let listenerCalled = false;

    watcher.addListener((_, event) => {
      if (listenerCalled) return;
      listenerCalled = true;

      expect(watcher.lastModified()).toBeGreaterThan(builtTime!);
      expect(event.changedPaths).toBeDefined();
      expect(Array.isArray(event.changedPaths)).toBe(true);
      expect(event.changedPaths.length).toBeGreaterThan(0);
      done();
    });

    setTimeout(() => {
      let attempts = 0;
      const tryModification = () => {
        if (listenerCalled) {
          return;
        }
        if (attempts >= MAX_RETRY_ATTEMPTS) {
          done(new Error("Listener was not called after maximum retry attempts"));
          return;
        }

        ++attempts;
        const newContent = `options:\n  key: value-${Date.now()}`;
        fs.writeFileSync(configPath, newContent);

        setTimeout(tryModification, RETRY_DELAY + attempts * 500);
      };

      tryModification();
    }, MODIFICATION_DEBOUNCE_MS + 100);
  }, MODIFICATION_DEBOUNCE_MS + 100 + RETRY_DELAY * MAX_RETRY_ATTEMPTS + MAX_RETRY_ATTEMPTS ** 2 * 500);

  // It would be nice to test this, but it's not reliable to watch on some operating systems.
  test("multiple listeners are all called", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);
    const builtTime = watcher.lastModified();

    let listener1Called = false;
    let listener2Called = false;
    let testCompleted = false;

    const checkAllListenersCalled = () => {
      if (listener1Called && listener2Called && !testCompleted) {
        testCompleted = true;
        done();
      }
    };

    // Add both listeners immediately
    watcher.addListener((_, event) => {
      if (!listener1Called) {
        listener1Called = true;
        expect(watcher.lastModified()).toBeGreaterThan(builtTime!);
        expect(event.changedPaths).toBeDefined();
        expect(Array.isArray(event.changedPaths)).toBe(true);
        expect(event.changedPaths.length).toBeGreaterThan(0);
        checkAllListenersCalled();
      }
    });

    watcher.addListener((_, event) => {
      if (!listener2Called) {
        listener2Called = true;
        expect(watcher.lastModified()).toBeGreaterThan(builtTime!);
        expect(event.changedPaths).toBeDefined();
        expect(Array.isArray(event.changedPaths)).toBe(true);
        expect(event.changedPaths.length).toBeGreaterThan(0);
        checkAllListenersCalled();
      }
    });

    setTimeout(() => {
      let attempts = 0;
      const tryModification = () => {
        if (testCompleted) {
          return;
        }
        if (attempts >= MAX_RETRY_ATTEMPTS) {
          const failureMsg = `Not all listeners were called after ${attempts} attempts. Listener1 called: ${listener1Called}, Listener2 called: ${listener2Called}`;
          done(new Error(failureMsg));
          return;
        }

        ++attempts;
        // Use a unique timestamp to ensure file content actually changes
        const newContent = `options:\n  key: value-${Date.now()}\n  attempt: ${attempts}`;
        fs.writeFileSync(configPath, newContent);

        setTimeout(tryModification, RETRY_DELAY + attempts * 200);
      };

      tryModification();
    }, MODIFICATION_DEBOUNCE_MS + 100);
  }, MODIFICATION_DEBOUNCE_MS + 100 + RETRY_DELAY * MAX_RETRY_ATTEMPTS + MAX_RETRY_ATTEMPTS ** 2 * 200);
});