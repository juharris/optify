import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { OptionsWatcher, OptionsWatcherListenerEvent } from '../index';

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
    watcher.addListener((_, event) => {
        expect(event.changedPaths).toBeDefined()
        expect(Array.isArray(event.changedPaths)).toBe(true);
        expect(event.changedPaths.length). toBeGreaterThan(0);
        done();
    });
    fs.writeFileSync(configPath, '');
    setTimeout(() => console.log("I/O time"),100);
    // Try again.
    fs.writeFileSync(configPath, '');
  }, 3000);

  test("multiple listeners are all called", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);

    let listener1Event: OptionsWatcherListenerEvent;
    let listener2Event: OptionsWatcherListenerEvent;

    const checkAllListenersCalled = () => {
      if (listener1Event && listener2Event) {
        done();
      }
    };

    watcher.addListener((_, event) => {
      listener1Event = event;
      checkAllListenersCalled();
    });

    watcher.addListener((_, event) => {
      listener2Event = event;
      checkAllListenersCalled();
    });

    fs.writeFileSync(configPath, '');
  }, 2000);
});