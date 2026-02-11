/**
 * Popup UI controller for WebTags extension
 */

import type { ExtensionState, BookmarksData } from '../types';

// UI state
let currentView: 'main' | 'setup' | 'settings' = 'setup';
let currentTab: 'bookmarks' | 'tags' = 'bookmarks';
let extensionState: ExtensionState | null = null;

// DOM elements
const elements = {
  statusBar: document.getElementById('status-bar')!,
  statusMessage: document.getElementById('status-message')!,
  mainView: document.getElementById('main-view')!,
  setupView: document.getElementById('setup-view')!,
  settingsView: document.getElementById('settings-view')!,

  syncBtn: document.getElementById('sync-btn')!,
  settingsBtn: document.getElementById('settings-btn')!,

  tabBookmarks: document.getElementById('tab-bookmarks')!,
  tabTags: document.getElementById('tab-tags')!,
  bookmarksContent: document.getElementById('bookmarks-content')!,
  tagsContent: document.getElementById('tags-content')!,

  searchInput: document.getElementById('search-input') as HTMLInputElement,
  bookmarksList: document.getElementById('bookmarks-list')!,
  tagsTree: document.getElementById('tags-tree')!,

  setupNewBtn: document.getElementById('setup-new-btn')!,
  setupCloneBtn: document.getElementById('setup-clone-btn')!,
  newRepoForm: document.getElementById('new-repo-form')!,
  cloneRepoForm: document.getElementById('clone-repo-form')!,

  authGithubBtn: document.getElementById('auth-github-btn')!,
  authStatus: document.getElementById('auth-status')!,
  createRepoBtn: document.getElementById('create-repo-btn')!,

  repoUrlInput: document.getElementById('repo-url-input') as HTMLInputElement,
  cloneRepoBtn: document.getElementById('clone-repo-btn')!,

  backToMainBtn: document.getElementById('back-to-main-btn')!,
  repoPath: document.getElementById('repo-path')!,
  lastSync: document.getElementById('last-sync')!,
  repoStatus: document.getElementById('repo-status')!,
  reauthGithubBtn: document.getElementById('reauth-github-btn')!,

  encryptionToggle: document.getElementById('encryption-toggle') as HTMLInputElement,
  encryptionStatusValue: document.getElementById('encryption-status-value')!,
  encryptionControls: document.getElementById('encryption-controls')!,
  encryptionNotSupported: document.getElementById('encryption-not-supported')!,

  createTagBtn: document.getElementById('create-tag-btn')!,
};

/**
 * Show status message
 */
function showStatus(message: string, type: 'info' | 'success' | 'error' = 'info'): void {
  elements.statusBar.className = `status-bar ${type}`;
  elements.statusMessage.textContent = message;
  elements.statusBar.classList.remove('hidden');

  setTimeout(() => {
    elements.statusBar.classList.add('hidden');
  }, 3000);
}

/**
 * Switch view
 */
function switchView(view: 'main' | 'setup' | 'settings'): void {
  currentView = view;

  elements.mainView.classList.toggle('hidden', view !== 'main');
  elements.setupView.classList.toggle('hidden', view !== 'setup');
  elements.settingsView.classList.toggle('hidden', view !== 'settings');
}

/**
 * Switch tab
 */
function switchTab(tab: 'bookmarks' | 'tags'): void {
  currentTab = tab;

  elements.tabBookmarks.classList.toggle('active', tab === 'bookmarks');
  elements.tabTags.classList.toggle('active', tab === 'tags');

  elements.bookmarksContent.classList.toggle('hidden', tab !== 'bookmarks');
  elements.tagsContent.classList.toggle('hidden', tab !== 'tags');
}

/**
 * Load extension state from background
 */
async function loadExtensionState(): Promise<void> {
  try {
    const response = await chrome.runtime.sendMessage({ type: 'getState' });
    extensionState = response;

    if (extensionState.initialized) {
      switchView('main');
      await loadBookmarks();
    } else {
      switchView('setup');
    }

    if (extensionState.error) {
      showStatus(extensionState.error, 'error');
    }
  } catch (error) {
    console.error('Failed to load extension state:', error);
    showStatus('Failed to connect to extension', 'error');
  }
}

/**
 * Load bookmarks
 */
async function loadBookmarks(): Promise<void> {
  try {
    const bookmarks = await chrome.bookmarks.getTree();
    renderBookmarks(bookmarks[0]);
  } catch (error) {
    console.error('Failed to load bookmarks:', error);
    showStatus('Failed to load bookmarks', 'error');
  }
}

/**
 * Render bookmarks
 */
function renderBookmarks(node: chrome.bookmarks.BookmarkTreeNode): void {
  elements.bookmarksList.innerHTML = '';

  const bookmarks = extractBookmarks(node);

  if (bookmarks.length === 0) {
    elements.bookmarksList.innerHTML = '<p style="padding: 16px; color: #999; text-align: center;">No bookmarks yet</p>';
    return;
  }

  for (const bookmark of bookmarks) {
    const item = createBookmarkElement(bookmark);
    elements.bookmarksList.appendChild(item);
  }
}

/**
 * Extract bookmarks from tree
 */
function extractBookmarks(node: chrome.bookmarks.BookmarkTreeNode): chrome.bookmarks.BookmarkTreeNode[] {
  const bookmarks: chrome.bookmarks.BookmarkTreeNode[] = [];

  if (node.url) {
    bookmarks.push(node);
  }

  if (node.children) {
    for (const child of node.children) {
      bookmarks.push(...extractBookmarks(child));
    }
  }

  return bookmarks;
}

/**
 * Create bookmark element
 */
function createBookmarkElement(bookmark: chrome.bookmarks.BookmarkTreeNode): HTMLElement {
  const item = document.createElement('div');
  item.className = 'bookmark-item';

  // Extract tags from title (using #tag syntax)
  const { cleanTitle, tags } = extractTagsFromTitle(bookmark.title);

  // Build DOM safely instead of using innerHTML with user data
  const titleDiv = document.createElement('div');
  titleDiv.className = 'bookmark-title';
  titleDiv.textContent = cleanTitle;
  item.appendChild(titleDiv);

  const urlDiv = document.createElement('div');
  urlDiv.className = 'bookmark-url';
  urlDiv.textContent = bookmark.url || '';
  item.appendChild(urlDiv);

  if (tags.length > 0) {
    const tagsDiv = document.createElement('div');
    tagsDiv.className = 'bookmark-tags';
    for (const tag of tags) {
      const tagChip = document.createElement('span');
      tagChip.className = 'tag-chip';
      tagChip.textContent = `#${tag}`;
      tagsDiv.appendChild(tagChip);
    }
    item.appendChild(tagsDiv);
  }

  item.addEventListener('click', () => {
    if (bookmark.url) {
      chrome.tabs.create({ url: bookmark.url });
    }
  });

  return item;
}

/**
 * Extract tags from title
 */
function extractTagsFromTitle(title: string): { cleanTitle: string; tags: string[] } {
  const tagRegex = /#(\w+)/g;
  const tags: string[] = [];
  let match;

  while ((match = tagRegex.exec(title)) !== null) {
    tags.push(match[1]);
  }

  const cleanTitle = title.replace(tagRegex, '').trim();

  return { cleanTitle, tags };
}

/**
 * Escape HTML
 */
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Manual sync
 */
async function performManualSync(): Promise<void> {
  showStatus('Syncing...', 'info');

  try {
    const response = await chrome.runtime.sendMessage({ type: 'manualSync' });

    if (response.success) {
      showStatus('Sync complete!', 'success');
      await loadBookmarks();
    } else {
      showStatus(response.error || 'Sync failed', 'error');
    }
  } catch (error) {
    console.error('Sync error:', error);
    showStatus('Sync failed', 'error');
  }
}

/**
 * Initialize repository with GitHub OAuth
 */
async function initWithGithub(): Promise<void> {
  showStatus('Authenticating with GitHub...', 'info');

  try {
    const response = await chrome.runtime.sendMessage({
      type: 'auth',
      method: 'oauth',
    });

    if (response.type === 'authflow') {
      // Open GitHub authorization page
      chrome.tabs.create({ url: response.verification_uri });

      showStatus(`Enter code: ${response.user_code}`, 'info');
      elements.authStatus.classList.remove('hidden');
    } else if (response.type === 'error') {
      showStatus(response.message, 'error');
    }
  } catch (error) {
    console.error('Auth error:', error);
    showStatus('Authentication failed', 'error');
  }
}

/**
 * Clone repository
 */
async function cloneRepository(): Promise<void> {
  const repoUrl = elements.repoUrlInput.value.trim();

  if (!repoUrl) {
    showStatus('Please enter a repository URL', 'error');
    return;
  }

  showStatus('Cloning repository...', 'info');

  try {
    const response = await chrome.runtime.sendMessage({
      type: 'init',
      repoUrl,
    });

    if (response.success) {
      showStatus('Repository cloned!', 'success');
      await loadExtensionState();
    } else {
      showStatus(response.error || 'Clone failed', 'error');
    }
  } catch (error) {
    console.error('Clone error:', error);
    showStatus('Clone failed', 'error');
  }
}

/**
 * Update settings view
 */
async function updateSettingsView(): Promise<void> {
  if (extensionState) {
    elements.repoStatus.textContent = extensionState.initialized ? 'Initialized' : 'Not initialized';

    if (extensionState.lastSyncTime) {
      const date = new Date(extensionState.lastSyncTime);
      elements.lastSync.textContent = date.toLocaleString();
    } else {
      elements.lastSync.textContent = 'Never';
    }
  }

  try {
    const response = await chrome.runtime.sendMessage({ type: 'getState' });
    extensionState = response;

    // Update encryption status
    await updateEncryptionStatus();
  } catch (error) {
    console.error('Failed to get state:', error);
  }
}

/**
 * Update encryption status in settings view
 */
async function updateEncryptionStatus(): Promise<void> {
  try {
    const response = await chrome.runtime.sendMessage({ type: 'encryptionStatus' });

    if (response.type === 'success' && response.data) {
      const { enabled, supported } = response.data;

      // Show/hide UI based on platform support
      if (!supported) {
        elements.encryptionControls.classList.add('hidden');
        elements.encryptionNotSupported.classList.remove('hidden');
        return;
      }

      elements.encryptionControls.classList.remove('hidden');
      elements.encryptionNotSupported.classList.add('hidden');

      // Update toggle state
      elements.encryptionToggle.checked = enabled;
      elements.encryptionToggle.disabled = false;

      // Update status text
      elements.encryptionStatusValue.textContent = enabled ? 'Enabled' : 'Disabled';

      // Update extension state
      if (extensionState) {
        extensionState.encryptionEnabled = enabled;
        extensionState.encryptionSupported = supported;
      }
    } else {
      elements.encryptionStatusValue.textContent = 'Unknown';
    }
  } catch (error) {
    console.error('Failed to check encryption status:', error);
    elements.encryptionStatusValue.textContent = 'Error';
  }
}

/**
 * Toggle encryption
 */
async function toggleEncryption(enable: boolean): Promise<void> {
  try {
    elements.encryptionToggle.disabled = true;
    elements.encryptionStatusValue.textContent = enable ? 'Enabling...' : 'Disabling...';

    const messageType = enable ? 'enableEncryption' : 'disableEncryption';
    const response = await chrome.runtime.sendMessage({ type: messageType });

    if (response.type === 'success') {
      showStatus(response.message, 'success');
      await updateEncryptionStatus();
    } else {
      showStatus(response.message, 'error');
      // Revert toggle state on error
      elements.encryptionToggle.checked = !enable;
      elements.encryptionToggle.disabled = false;
    }
  } catch (error) {
    console.error('Failed to toggle encryption:', error);
    showStatus('Failed to toggle encryption', 'error');
    elements.encryptionToggle.checked = !enable;
    elements.encryptionToggle.disabled = false;
  }
}

// Event listeners
elements.syncBtn.addEventListener('click', performManualSync);
elements.settingsBtn.addEventListener('click', () => {
  switchView('settings');
  updateSettingsView();
});

elements.tabBookmarks.addEventListener('click', () => switchTab('bookmarks'));
elements.tabTags.addEventListener('click', () => switchTab('tags'));

elements.setupNewBtn.addEventListener('click', () => {
  elements.newRepoForm.classList.remove('hidden');
  elements.cloneRepoForm.classList.add('hidden');
});

elements.setupCloneBtn.addEventListener('click', () => {
  elements.cloneRepoForm.classList.remove('hidden');
  elements.newRepoForm.classList.add('hidden');
});

elements.authGithubBtn.addEventListener('click', initWithGithub);
elements.cloneRepoBtn.addEventListener('click', cloneRepository);

elements.backToMainBtn.addEventListener('click', () => switchView('main'));

elements.encryptionToggle.addEventListener('change', (e) => {
  const enabled = (e.target as HTMLInputElement).checked;
  toggleEncryption(enabled);
});

elements.searchInput.addEventListener('input', (e) => {
  const query = (e.target as HTMLInputElement).value.toLowerCase();
  // TODO: Implement search filtering
  console.log('Search:', query);
});

// Initialize
loadExtensionState();
