import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { OptionsWatcher, OptionsWatcherListenerEvent } from '../index';

const MODIFICATION_DEBOUNCE_MS = 1000;
const RETRY_DELAY = MODIFICATION_DEBOUNCE_MS + 500;
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
    let listenerCalled = false;

    watcher.addListener((_, event) => {
      if (listenerCalled) return;
      listenerCalled = true;

      expect(event.changedPaths).toBeDefined();
      expect(Array.isArray(event.changedPaths)).toBe(true);
      expect(event.changedPaths.length).toBeGreaterThan(0);
      done();
    });

    setTimeout(() => {
      let attempts = 0;
      const tryModification = () => {
        if (listenerCalled || attempts >= MAX_RETRY_ATTEMPTS) {
          if (!listenerCalled && attempts >= MAX_RETRY_ATTEMPTS) {
            done(new Error('Listener was not called after maximum retry attempts'));
          }
          return;
        }

        ++attempts;
        fs.writeFileSync(configPath, 'options:\n  key: value');

        setTimeout(tryModification, RETRY_DELAY);
      };

      tryModification();
    }, RETRY_DELAY);
  }, RETRY_DELAY * MAX_RETRY_ATTEMPTS + MODIFICATION_DEBOUNCE_MS + 1000);

  test("multiple listeners are all called", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);

    let listener1Event: OptionsWatcherListenerEvent | null = null;
    let listener2Event: OptionsWatcherListenerEvent | null = null;
    let allListenersCalled = false;

    const checkAllListenersCalled = () => {
      if (listener1Event && listener2Event && !allListenersCalled) {
        allListenersCalled = true;
        done();
      }
    };

    watcher.addListener((_, event) => {
      if (!listener1Event) {
        listener1Event = event;
        expect(event.changedPaths).toBeDefined();
        expect(Array.isArray(event.changedPaths)).toBe(true);
        expect(event.changedPaths.length).toBeGreaterThan(0);
        checkAllListenersCalled();
      }
    });

    watcher.addListener((_, event) => {
      if (!listener2Event) {
        listener2Event = event;
        checkAllListenersCalled();
      }
    });

    setTimeout(() => {
      let attempts = 0;
      const tryModification = () => {
        if (allListenersCalled || attempts >= MAX_RETRY_ATTEMPTS) {
          if (!allListenersCalled && attempts >= MAX_RETRY_ATTEMPTS) {
            done(new Error('Not all listeners were called after maximum retry attempts'));
          }
          return;
        }

        attempts++;
        fs.writeFileSync(configPath, `key: value-${Date.now()}`);

        setTimeout(tryModification, RETRY_DELAY);
      };

      tryModification();
    }, RETRY_DELAY);
  }, RETRY_DELAY * MAX_RETRY_ATTEMPTS + MODIFICATION_DEBOUNCE_MS + 1000);
});