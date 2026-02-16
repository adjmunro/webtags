import { test, expect, chromium, BrowserContext, Page, _electron as electron } from '@playwright/test';
import path from 'path';
import os from 'os';
import fs from 'fs';

const EXTENSION_PATH = path.join(__dirname, '../../dist');
const NATIVE_HOST_NAME = 'com.webtags.host';
// Use a consistent profile directory
const USER_DATA_DIR = path.join(os.homedir(), 'Library', 'Application Support', 'Playwright', 'webtags-test-profile');

// Helper to get extension service worker
async function getServiceWorker(context: BrowserContext) {
  // Try to get existing service workers first
  let workers = context.serviceWorkers();

  if (workers.length === 0) {
    // Wait for service worker to register
    await context.waitForEvent('serviceworker', { timeout: 10000 }).catch(() => null);
    workers = context.serviceWorkers();
  }

  return workers.find(w => w.url().includes('chrome-extension://'));
}

// Helper to get extension ID from service worker URL
function getExtensionIdFromWorker(context: BrowserContext): string | null {
  const workers = context.serviceWorkers();
  for (const worker of workers) {
    const url = worker.url();
    if (url.includes('chrome-extension://')) {
      const match = url.match(/chrome-extension:\/\/([a-z]+)\//);
      if (match) {
        return match[1];
      }
    }
  }
  return null;
}

// Helper to send native message via popup page
async function sendNativeMessage(context: BrowserContext, message: any) {
  // Get extension ID
  const extensionId = getExtensionIdFromWorker(context);
  if (!extensionId) {
    throw new Error('Extension ID not found');
  }

  // Open popup page
  const popupUrl = `chrome-extension://${extensionId}/popup.html`;
  const page = await context.newPage();
  await page.goto(popupUrl);

  // Execute sendNativeMessage from popup context
  const result = await page.evaluate(async ([hostName, msg]) => {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendNativeMessage(
        hostName,
        msg,
        (response: any) => {
          if (chrome.runtime.lastError) {
            reject(JSON.stringify(chrome.runtime.lastError));
          } else {
            resolve(response);
          }
        }
      );
    });
  }, [NATIVE_HOST_NAME, message]);

  await page.close();
  return result;
}

test.describe('OAuth Authentication', () => {
  test('should connect to native host and get status', async () => {
    // Launch browser with extension and persistent profile
    const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
      headless: false,
      args: [
        `--disable-extensions-except=${EXTENSION_PATH}`,
        `--load-extension=${EXTENSION_PATH}`,
      ],
    });

    // Wait a bit for extension to load
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Get and display extension ID
    const extensionId = getExtensionIdFromWorker(context);
    console.log('Extension ID:', extensionId);
    console.log('\nIMPORTANT: Update native messaging manifest with this extension ID:');
    console.log(`  "allowed_origins": ["chrome-extension://${extensionId}/"]`);
    console.log(`\nRun this command:`);
    console.log(`  scripts/setup-manifests.sh ${extensionId}\n`);

    // Test native host status check
    console.log('Testing native host connection...');
    try {
      const statusResult = await sendNativeMessage(context, { type: 'status' });
      console.log('Status check result:', statusResult);
      expect(statusResult).toBeDefined();
    } catch (error) {
      console.error('Native host connection failed:', error);
      console.log('\nMake sure to:');
      console.log('1. Update the native messaging manifest with the extension ID above');
      console.log('2. Ensure webtags-host is installed: /opt/homebrew/bin/webtags-host');
      throw error;
    }

    await context.close();
  });

  test('should initiate OAuth device flow', async () => {
    // Launch browser with extension and persistent profile
    const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
      headless: false,
      args: [
        `--disable-extensions-except=${EXTENSION_PATH}`,
        `--load-extension=${EXTENSION_PATH}`,
      ],
    });

    // Test OAuth authentication
    console.log('Initiating OAuth device flow...');
    const oauthResult = await sendNativeMessage(context, {
      type: 'auth',
      method: 'oauth'
    });

    console.log('OAuth result:', oauthResult);
    expect(oauthResult).toBeDefined();

    // If we get device flow response, display the verification URL
    if (oauthResult && typeof oauthResult === 'object' && 'verification_uri' in oauthResult) {
      console.log('\n=== OAuth Device Flow ===');
      console.log('Verification URL:', oauthResult.verification_uri);
      console.log('User Code:', oauthResult.user_code);
      console.log('Please visit the URL above and enter the code to complete authentication');
      console.log('========================\n');

      // Keep browser open for manual verification
      console.log('Waiting 120 seconds for you to complete OAuth flow...');
      const page = await context.newPage();
      await page.waitForTimeout(120000);
    }

    await context.close();
  });

  test('should initialize repository', async () => {
    // This test assumes OAuth is already complete
    const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
      headless: false,
      args: [
        `--disable-extensions-except=${EXTENSION_PATH}`,
        `--load-extension=${EXTENSION_PATH}`,
      ],
    });

    // Get username from environment or use placeholder
    const username = process.env.GITHUB_USERNAME || 'adjmunro';
    const repoUrl = `https://github.com/${username}/webtags-bookmarks.git`;

    console.log(`Attempting to initialize repository: ${repoUrl}`);
    console.log('Make sure this repository exists first!');

    const initResult = await sendNativeMessage(context, {
      type: 'init',
      repoUrl: repoUrl
    });

    console.log('Init result:', initResult);
    expect(initResult).toBeDefined();

    await context.close();
  });

  test('should sync bookmarks', async () => {
    // This test assumes OAuth and init are complete
    const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
      headless: false,
      args: [
        `--disable-extensions-except=${EXTENSION_PATH}`,
        `--load-extension=${EXTENSION_PATH}`,
      ],
    });

    const testData = {
      bookmarks: [{
        id: '1',
        url: 'https://example.com',
        title: 'Test Bookmark',
        tags: ['test']
      }],
      tags: [{ name: 'test', color: '#0000ff' }]
    };

    console.log('Attempting to sync bookmarks:', testData);

    const syncResult = await sendNativeMessage(context, {
      type: 'write',
      data: testData
    });

    console.log('Sync result:', syncResult);
    expect(syncResult).toBeDefined();

    await context.close();
  });
});
