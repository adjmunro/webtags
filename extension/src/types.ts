/**
 * TypeScript types for WebTags extension
 */

// JSON API v1.1 types

export interface JsonApiVersion {
  version: string;
}

export type Resource = BookmarkResource | TagResource;

export interface BookmarkResource {
  type: 'bookmark';
  id: string;
  attributes: BookmarkAttributes;
  relationships?: BookmarkRelationships;
}

export interface BookmarkAttributes {
  url: string;
  title: string;
  created: string; // ISO 8601 datetime
  modified?: string; // ISO 8601 datetime
  notes?: string;
}

export interface BookmarkRelationships {
  tags?: RelationshipData;
}

export interface RelationshipData {
  data: ResourceIdentifier[];
}

export interface ResourceIdentifier {
  type: string;
  id: string;
}

export interface TagResource {
  type: 'tag';
  id: string;
  attributes: TagAttributes;
  relationships?: TagRelationships;
}

export interface TagAttributes {
  name: string;
  color?: string; // Hex color (e.g., "#3b82f6")
  description?: string;
}

export interface TagRelationships {
  parent?: ParentRelationship;
}

export interface ParentRelationship {
  data: ResourceIdentifier | null;
}

export interface BookmarksData {
  jsonapi: JsonApiVersion;
  data: Resource[];
  included?: Resource[];
}

// Native messaging protocol types

export type NativeMessage =
  | InitMessage
  | WriteMessage
  | ReadMessage
  | SyncMessage
  | AuthMessage
  | StatusMessage
  | EnableEncryptionMessage
  | DisableEncryptionMessage
  | EncryptionStatusMessage;

export interface InitMessage {
  type: 'init';
  repo_path?: string;
  repo_url?: string;
}

export interface WriteMessage {
  type: 'write';
  data: BookmarksData;
}

export interface ReadMessage {
  type: 'read';
}

export interface SyncMessage {
  type: 'sync';
}

export interface AuthMessage {
  type: 'auth';
  method: 'oauth' | 'pat';
  token?: string;
}

export interface StatusMessage {
  type: 'status';
}

export interface EnableEncryptionMessage {
  type: 'enableencryption';
}

export interface DisableEncryptionMessage {
  type: 'disableencryption';
}

export interface EncryptionStatusMessage {
  type: 'encryptionstatus';
}

export type NativeResponse = SuccessResponse | ErrorResponse | AuthFlowResponse;

export interface SuccessResponse {
  type: 'success';
  message: string;
  data?: any;
}

export interface ErrorResponse {
  type: 'error';
  message: string;
  code?: string;
}

export interface AuthFlowResponse {
  type: 'authflow';
  user_code: string;
  verification_uri: string;
  device_code: string;
}

// Browser bookmark types

export interface ChromeBookmark {
  id: string;
  parentId?: string;
  index?: number;
  url?: string;
  title: string;
  dateAdded?: number;
  dateGroupModified?: number;
  children?: ChromeBookmark[];
}

// Extension state types

export interface ExtensionState {
  initialized: boolean;
  syncing: boolean;
  lastSyncTime?: number;
  error?: string;
  encryptionEnabled?: boolean;
  encryptionSupported?: boolean;
}

// Tag hierarchy helper types

export interface TagNode {
  tag: TagResource;
  children: TagNode[];
  parent?: TagNode;
}

export interface TagBreadcrumb {
  id: string;
  name: string;
}
