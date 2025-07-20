import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { OptionsWatcher } from '../index';

describe("OptionsWatcher", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-watcher-test-'));
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test("listener is called when a file is added", (done) => {
    const watcher = OptionsWatcher.build(tempDir);

    watcher.addListener(() => done());
    const newConfigPath = path.join(tempDir, 'new_feature.yaml');
    fs.writeFileSync(newConfigPath, '');
  }, 2000);

  test("listener is called when a file is modified", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);
    watcher.addListener(() => done());
    fs.writeFileSync(configPath, '');
  }, 2000);

  test("multiple listeners are all called", (done) => {
    const configPath = path.join(tempDir, 'config.yaml');
    fs.writeFileSync(configPath, '');

    const watcher = OptionsWatcher.build(tempDir);

    let listener1Called = false;
    let listener2Called = false;

    watcher.addListener(() => {
      listener1Called = true;
      checkAllListenersCalled();
    });

    watcher.addListener(() => {
      listener2Called = true;
      checkAllListenersCalled();
    });

    const checkAllListenersCalled = () => {
      if (listener1Called && listener2Called) {
        done();
      }
    };

    fs.writeFileSync(configPath, '');
  }, 2000);
});