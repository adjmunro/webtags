/**
 * Native messaging client for communicating with the Rust native host
 */

import type {
  NativeMessage,
  NativeResponse,
  BookmarksData,
  AuthMessage,
} from './types';

const NATIVE_HOST_NAME = 'com.webtags.host';

export class NativeMessagingClient {
  private port: chrome.runtime.Port | null = null;
  private messageQueue: NativeMessage[] = [];
  private connected = false;
  private reconnectTimeout: number | null = null;
  private responseHandlers = new Map<
    number,
    {
      resolve: (response: NativeResponse) => void;
      reject: (error: Error) => void;
    }
  >();
  private messageIdCounter = 0;

  constructor() {
    this.connect();
  }

  /**
   * Connect to the native messaging host
   */
  private connect(): void {
    try {
      this.port = chrome.runtime.connectNative(NATIVE_HOST_NAME);

      this.port.onMessage.addListener((message: any) => {
        this.handleResponse(message);
      });

      this.port.onDisconnect.addListener(() => {
        this.handleDisconnect();
      });

      this.connected = true;
      console.log('Connected to native host');

      // Send queued messages
      this.flushQueue();
    } catch (error) {
      console.error('Failed to connect to native host:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * Handle disconnection from native host
   */
  private handleDisconnect(): void {
    this.connected = false;
    this.port = null;

    const lastError = chrome.runtime.lastError;
    if (lastError) {
      console.error('Native host disconnected:', lastError.message);
    }

    // Reject all pending promises
    for (const handler of this.responseHandlers.values()) {
      handler.reject(new Error('Native host disconnected'));
    }
    this.responseHandlers.clear();

    // Schedule reconnect
    this.scheduleReconnect();
  }

  /**
   * Schedule a reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimeout !== null) {
      return;
    }

    this.reconnectTimeout = window.setTimeout(() => {
      this.reconnectTimeout = null;
      this.connect();
    }, 5000); // Retry after 5 seconds
  }

  /**
   * Handle response from native host
   */
  private handleResponse(response: NativeResponse): void {
    console.log('Received response from native host:', response);

    // For now, resolve the most recent pending handler
    // In a real implementation, we'd use message IDs to match requests/responses
    const handler = Array.from(this.responseHandlers.values())[0];
    if (handler) {
      this.responseHandlers.clear();
      handler.resolve(response);
    }
  }

  /**
   * Send a message to the native host
   */
  private async sendMessage(message: NativeMessage): Promise<NativeResponse> {
    return new Promise((resolve, reject) => {
      if (!this.connected || !this.port) {
        // Queue the message for later
        this.messageQueue.push(message);
        reject(new Error('Not connected to native host'));
        return;
      }

      try {
        const messageId = this.messageIdCounter++;
        this.responseHandlers.set(messageId, { resolve, reject });

        this.port.postMessage(message);
      } catch (error) {
        reject(error);
      }

      // Set timeout for response
      setTimeout(() => {
        const handler = Array.from(this.responseHandlers.values())[0];
        if (handler) {
          this.responseHandlers.clear();
          handler.reject(new Error('Request timeout'));
        }
      }, 30000); // 30 second timeout
    });
  }

  /**
   * Flush queued messages
   */
  private flushQueue(): void {
    while (this.messageQueue.length > 0 && this.connected) {
      const message = this.messageQueue.shift();
      if (message) {
        this.sendMessage(message).catch((error) => {
          console.error('Failed to send queued message:', error);
        });
      }
    }
  }

  /**
   * Initialize the repository
   */
  async init(repoPath?: string, repoUrl?: string): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'init',
      repo_path: repoPath,
      repo_url: repoUrl,
    });
  }

  /**
   * Write bookmarks data to file and sync
   */
  async write(data: BookmarksData): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'write',
      data,
    });
  }

  /**
   * Read bookmarks data from file
   */
  async read(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'read',
    });
  }

  /**
   * Sync with remote repository
   */
  async sync(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'sync',
    });
  }

  /**
   * Authenticate with GitHub
   */
  async auth(method: 'oauth' | 'pat', token?: string): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'auth',
      method,
      token,
    });
  }

  /**
   * Get repository status
   */
  async status(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'status',
    });
  }

  /**
   * Enable encryption
   */
  async enableEncryption(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'enableencryption',
    });
  }

  /**
   * Disable encryption
   */
  async disableEncryption(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'disableencryption',
    });
  }

  /**
   * Get encryption status
   */
  async encryptionStatus(): Promise<NativeResponse> {
    return this.sendMessage({
      type: 'encryptionstatus',
    });
  }

  /**
   * Check if connected to native host
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Disconnect from native host
   */
  disconnect(): void {
    if (this.reconnectTimeout !== null) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    if (this.port) {
      this.port.disconnect();
      this.port = null;
    }

    this.connected = false;
  }
}

// Singleton instance
export const nativeClient = new NativeMessagingClient();
