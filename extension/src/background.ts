/**
 * Background service worker for WebTags extension
 * Handles bookmark synchronization and native messaging
 */

import { nativeClient } from './messaging';
import {
  exportChromeBookmarks,
  applyToChrome,
} from './bookmarkConverter';
import type { ExtensionState, BookmarksData, NativeResponse } from './types';

// Extension state
const state: ExtensionState = {
  initialized: false,
  syncing: false,
  lastSyncTime: undefined,
  error: undefined,
};

// Sync interval (1 hour)
const SYNC_INTERVAL = 60 * 60 * 1000;

/**
 * Initialize the extension
 */
async function initialize(): Promise<void> {
  console.log('Initializing WebTags extension...');

  try {
    // Check if native host is connected
    if (!nativeClient.isConnected()) {
      console.warn('Native host not connected');
      state.error = 'Native host not connected';
      return;
    }

    // Get status from native host
    const statusResponse = await nativeClient.status();

    if (statusResponse.type === 'error') {
      console.error('Failed to get status:', statusResponse.message);
      state.error = statusResponse.message;
      return;
    }

    if (statusResponse.type === 'success' && statusResponse.data) {
      if (statusResponse.data.initialized) {
        console.log('Repository already initialized');
        state.initialized = true;

        // Perform initial sync
        await performSync();
      } else {
        console.log('Repository not initialized');
        state.initialized = false;
      }
    }
  } catch (error) {
    console.error('Initialization error:', error);
    state.error = error instanceof Error ? error.message : 'Unknown error';
  }
}

/**
 * Perform synchronization with remote repository
 */
async function performSync(): Promise<void> {
  if (state.syncing) {
    console.log('Sync already in progress');
    return;
  }

  state.syncing = true;
  state.error = undefined;

  try {
    console.log('Starting sync...');

    // Pull from remote
    const syncResponse = await nativeClient.sync();

    if (syncResponse.type === 'error') {
      console.error('Sync failed:', syncResponse.message);
      state.error = syncResponse.message;
      return;
    }

    // Read updated bookmarks data
    const readResponse = await nativeClient.read();

    if (readResponse.type === 'error') {
      console.error('Failed to read bookmarks:', readResponse.message);
      state.error = readResponse.message;
      return;
    }

    if (readResponse.type === 'success' && readResponse.data) {
      const bookmarksData = readResponse.data as BookmarksData;

      // Apply to Chrome bookmarks
      await applyToChrome(bookmarksData);

      console.log('Sync completed successfully');
      state.lastSyncTime = Date.now();
    }
  } catch (error) {
    console.error('Sync error:', error);
    state.error = error instanceof Error ? error.message : 'Unknown error';
  } finally {
    state.syncing = false;
  }
}

/**
 * Export bookmarks and push to remote
 */
async function exportAndPush(): Promise<void> {
  if (state.syncing) {
    console.log('Sync in progress, skipping export');
    return;
  }

  state.syncing = true;

  try {
    console.log('Exporting bookmarks...');

    // Export Chrome bookmarks
    const bookmarksData = await exportChromeBookmarks();

    // Write to file and push
    const writeResponse = await nativeClient.write(bookmarksData);

    if (writeResponse.type === 'error') {
      console.error('Failed to write bookmarks:', writeResponse.message);
      state.error = writeResponse.message;
      return;
    }

    console.log('Bookmarks exported and pushed successfully');
    state.lastSyncTime = Date.now();
  } catch (error) {
    console.error('Export error:', error);
    state.error = error instanceof Error ? error.message : 'Unknown error';
  } finally {
    state.syncing = false;
  }
}

/**
 * Handle bookmark created event
 */
function onBookmarkCreated(id: string, bookmark: chrome.bookmarks.BookmarkTreeNode): void {
  console.log('Bookmark created:', id, bookmark);
  exportAndPush();
}

/**
 * Handle bookmark removed event
 */
function onBookmarkRemoved(id: string, removeInfo: chrome.bookmarks.BookmarkRemoveInfo): void {
  console.log('Bookmark removed:', id, removeInfo);
  exportAndPush();
}

/**
 * Handle bookmark changed event
 */
function onBookmarkChanged(id: string, changeInfo: chrome.bookmarks.BookmarkChangeInfo): void {
  console.log('Bookmark changed:', id, changeInfo);
  exportAndPush();
}

/**
 * Handle bookmark moved event
 */
function onBookmarkMoved(id: string, moveInfo: chrome.bookmarks.BookmarkMoveInfo): void {
  console.log('Bookmark moved:', id, moveInfo);
  exportAndPush();
}

/**
 * Handle messages from popup or content scripts
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('Received message:', message);

  if (message.type === 'getState') {
    sendResponse(state);
    return true;
  }

  if (message.type === 'manualSync') {
    performSync().then(() => {
      sendResponse({ success: true });
    }).catch((error) => {
      sendResponse({ success: false, error: error.message });
    });
    return true; // Async response
  }

  if (message.type === 'init') {
    const { repoPath, repoUrl } = message;
    nativeClient.init(repoPath, repoUrl).then((response) => {
      if (response.type === 'success') {
        state.initialized = true;
        performSync().then(() => {
          sendResponse({ success: true });
        });
      } else if (response.type === 'error') {
        sendResponse({ success: false, error: response.message });
      } else {
        sendResponse({ success: false, error: 'Unexpected response type' });
      }
    }).catch((error) => {
      sendResponse({ success: false, error: error.message });
    });
    return true; // Async response
  }

  if (message.type === 'auth') {
    const { method, token } = message;
    nativeClient.auth(method, token).then((response) => {
      sendResponse(response);
    }).catch((error) => {
      sendResponse({ type: 'error', message: error.message });
    });
    return true; // Async response
  }

  if (message.type === 'enableEncryption') {
    nativeClient.enableEncryption().then((response) => {
      if (response.type === 'success') {
        state.encryptionEnabled = true;
      }
      sendResponse(response);
    }).catch((error) => {
      sendResponse({ type: 'error', message: error.message });
    });
    return true; // Async response
  }

  if (message.type === 'disableEncryption') {
    nativeClient.disableEncryption().then((response) => {
      if (response.type === 'success') {
        state.encryptionEnabled = false;
      }
      sendResponse(response);
    }).catch((error) => {
      sendResponse({ type: 'error', message: error.message });
    });
    return true; // Async response
  }

  if (message.type === 'encryptionStatus') {
    nativeClient.encryptionStatus().then((response) => {
      if (response.type === 'success' && response.data) {
        state.encryptionEnabled = response.data.enabled;
        state.encryptionSupported = response.data.supported;
      }
      sendResponse(response);
    }).catch((error) => {
      sendResponse({ type: 'error', message: error.message });
    });
    return true; // Async response
  }

  return false;
});

/**
 * Set up bookmark event listeners
 */
function setupBookmarkListeners(): void {
  chrome.bookmarks.onCreated.addListener(onBookmarkCreated);
  chrome.bookmarks.onRemoved.addListener(onBookmarkRemoved);
  chrome.bookmarks.onChanged.addListener(onBookmarkChanged);
  chrome.bookmarks.onMoved.addListener(onBookmarkMoved);
}

/**
 * Set up periodic sync
 */
function setupPeriodicSync(): void {
  setInterval(() => {
    if (state.initialized && !state.syncing) {
      const timeSinceLastSync = Date.now() - (state.lastSyncTime || 0);
      if (timeSinceLastSync >= SYNC_INTERVAL) {
        console.log('Performing periodic sync...');
        performSync();
      }
    }
  }, SYNC_INTERVAL);
}

// Initialize on startup
console.log('WebTags background service worker starting...');
initialize();
setupBookmarkListeners();
setupPeriodicSync();
