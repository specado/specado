/**
 * Basic functionality tests for Specado Node.js bindings
 */

import { init, getVersion, getVersionInfo } from '../index';

describe('Basic Functionality', () => {
  test('should initialize successfully', () => {
    const result = init();
    expect(result).toBe('Specado Node.js bindings initialized');
  });

  test('should return version string', () => {
    const version = getVersion();
    expect(typeof version).toBe('string');
    expect(version).toMatch(/specado-nodejs/);
  });

  test('should return detailed version info', () => {
    const versionInfo = getVersionInfo();
    
    expect(versionInfo).toHaveProperty('nodejsBinding');
    expect(versionInfo).toHaveProperty('coreLibrary');
    expect(versionInfo).toHaveProperty('buildTimestamp');
    expect(versionInfo).toHaveProperty('gitCommit');
    
    expect(typeof versionInfo.nodejsBinding).toBe('string');
    expect(typeof versionInfo.coreLibrary).toBe('string');
    expect(typeof versionInfo.buildTimestamp).toBe('string');
    expect(typeof versionInfo.gitCommit).toBe('string');
  });
});