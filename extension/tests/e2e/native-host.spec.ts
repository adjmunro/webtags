import { test, expect } from '@playwright/test';
import { spawn } from 'child_process';

const NATIVE_HOST_PATH = process.env.WEBTAGS_HOST_PATH || '/opt/homebrew/bin/webtags-host';

// Helper to send message to native host
async function sendNativeMessage(message: any): Promise<any> {
  return new Promise((resolve, reject) => {
    const nativeHost = spawn(NATIVE_HOST_PATH);
    const responseChunks: Buffer[] = [];
    let resolved = false;

    nativeHost.stdout.on('data', (data: Buffer) => {
      responseChunks.push(data);

      // Try to parse response after each chunk
      if (!resolved) {
        const responseBuffer = Buffer.concat(responseChunks);
        if (responseBuffer.length >= 4) {
          const messageLength = responseBuffer.readUInt32LE(0);
          if (responseBuffer.length >= 4 + messageLength) {
            try {
              const messageData = responseBuffer.slice(4, 4 + messageLength);
              const response = JSON.parse(messageData.toString('utf8'));
              resolved = true;
              nativeHost.kill();
              resolve(response);
            } catch (error) {
              console.error('Failed to parse response:', error);
            }
          }
        }
      }
    });

    nativeHost.stderr.on('data', (data) => {
      console.error('Native host stderr:', data.toString());
    });

    nativeHost.on('error', (error) => {
      if (!resolved) {
        reject(error);
      }
    });

    nativeHost.on('close', (code) => {
      if (!resolved) {
        reject(new Error(`Native host exited with code ${code} before sending response`));
      }
    });

    // Send message in native messaging format
    const messageJson = JSON.stringify(message);
    const messageLength = Buffer.byteLength(messageJson);
    const lengthBuffer = Buffer.alloc(4);
    lengthBuffer.writeUInt32LE(messageLength, 0);

    nativeHost.stdin.write(lengthBuffer);
    nativeHost.stdin.write(messageJson);
    nativeHost.stdin.end();

    // Set timeout
    setTimeout(() => {
      if (!resolved) {
        nativeHost.kill();
        reject(new Error('Native host response timeout'));
      }
    }, 10000);
  });
}

test.describe('Native Host Direct Testing', () => {
  test('should respond to status check', async () => {
    const response = await sendNativeMessage({ type: 'status' });
    console.log('Status response:', response);
    expect(response).toBeDefined();
    expect(response.type).toBe('success');
    expect(response.data).toBeDefined();
    expect(response.data.initialized).toBeDefined();
  });

  test('should initiate OAuth device flow', async () => {
    const response = await sendNativeMessage({
      type: 'auth',
      method: 'oauth'
    });

    console.log('OAuth response:', response);
    expect(response).toBeDefined();

    if (response.verification_uri) {
      console.log('\n=== OAuth Device Flow ===');
      console.log('Verification URL:', response.verification_uri);
      console.log('User Code:', response.user_code);
      console.log('Please visit the URL above and enter the code to complete authentication');
      console.log('========================\n');
    }
  });

  test('should initialize repository', async () => {
    const username = process.env.GITHUB_USERNAME || 'adjmunro';
    const repoUrl = `https://github.com/${username}/webtags-bookmarks.git`;

    console.log(`Attempting to initialize repository: ${repoUrl}`);

    const response = await sendNativeMessage({
      type: 'init',
      repoUrl: repoUrl
    });

    console.log('Init response:', response);
    expect(response).toBeDefined();
  });

  test('should sync bookmarks', async () => {
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

    const response = await sendNativeMessage({
      type: 'write',
      data: testData
    });

    console.log('Sync response:', response);
    expect(response).toBeDefined();
  });
});
